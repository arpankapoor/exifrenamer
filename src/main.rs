use anyhow::{anyhow, Context};
use exif::DateTime;
use exif::{In, Tag, Value};
use glob::{glob_with, MatchOptions};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::PathBuf;

struct ImageFile {
    path: PathBuf,
    datetime: DateTime,
}

impl TryFrom<PathBuf> for ImageFile {
    type Error = anyhow::Error;
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut bufreader = BufReader::new(File::open(path.as_path())?);
        let exifreader = exif::Reader::new();
        let exif = exifreader
            .read_from_container(&mut bufreader)
            .with_context(|| format!("unable to read exif in {}", path.display()))?;
        let datetime =
            match exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
                Some(field) => match &field.value {
                    Value::Ascii(s) => DateTime::from_ascii(s.first().with_context(|| {
                        format!("date field value missing in {}", path.display())
                    })?),
                    _ => Err(anyhow!("invalid date field in {}", path.display()))?,
                },
                None => Err(anyhow!("missing date field in {}", path.display()))?,
            }?;

        Ok(Self { path, datetime })
    }
}

impl Ord for ImageFile {
    fn cmp(&self, other: &Self) -> Ordering {
        let sdt = &self.datetime;
        let odt = &other.datetime;
        (
            sdt.year, sdt.month, sdt.day, sdt.hour, sdt.minute, sdt.second, &self.path,
        )
            .cmp(&(
                odt.year,
                odt.month,
                odt.day,
                odt.hour,
                odt.minute,
                odt.second,
                &other.path,
            ))
    }
}

impl PartialOrd for ImageFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ImageFile {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for ImageFile {}

impl Hash for ImageFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        let dt = &self.datetime;
        (dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second).hash(state);
    }
}

fn main() -> anyhow::Result<()> {
    let base_dir = std::env::args()
        .nth(1)
        .with_context(|| format!("usage: {} BASE_DIRECTORY", env!("CARGO_BIN_NAME")))?;

    let options = MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    let image_paths = glob_with(&format!("{}/**/*.jpg", base_dir), options)?
        .chain(glob_with(&format!("{}/**/*.png", base_dir), options)?)
        .collect::<Result<Vec<_>, _>>()?;

    let mut image_files = image_paths
        .into_iter()
        .map(|p| p.try_into())
        .collect::<Result<Vec<ImageFile>, _>>()?;

    image_files.sort();

    let years: HashSet<u16> = image_files.iter().map(|imf| imf.datetime.year).collect();

    let tmp_dir = tempfile::Builder::new()
        .prefix("organized")
        .tempdir_in(base_dir)?
        .into_path();

    for year in years {
        std::fs::create_dir(tmp_dir.join(year.to_string()))?;
    }

    let new_names_map = image_files
        .into_iter()
        .scan(
            (1u32, None::<DateTime>),
            |(prev_idx, prev_dt_option), imf| {
                let curr_dt = &imf.datetime;
                let mut curr_idx = 1;
                if let Some(prev_dt) = prev_dt_option {
                    if (curr_dt.year, curr_dt.month, curr_dt.day)
                        == (prev_dt.year, prev_dt.month, prev_dt.day)
                    {
                        curr_idx = *prev_idx + 1;
                    }
                }

                *prev_idx = curr_idx;
                *prev_dt_option = Some(DateTime { ..*curr_dt });

                let new_path = tmp_dir.join(format!(
                    "{:04}/{:04}-{:02}-{:02}_{:05}.{}",
                    curr_dt.year,
                    curr_dt.year,
                    curr_dt.month,
                    curr_dt.day,
                    curr_idx,
                    imf.path.extension().unwrap().to_str().unwrap()
                ));

                Some((imf, new_path))
            },
        )
        .collect::<HashMap<_, _>>();

    for (imf, new_path) in new_names_map {
        println!("{} -> {}", imf.path.display(), new_path.display());
        std::fs::rename(imf.path, new_path)?;
    }

    Ok(())
}
