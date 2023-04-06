use anyhow::{bail, Context};
use exif::{DateTime, In, Tag, Value};
use glob::{glob_with, MatchOptions};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::{File, Permissions};
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

struct ImageFile {
    path: PathBuf,
    datetime: OffsetDateTime,
}

impl TryFrom<PathBuf> for ImageFile {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut bufreader = BufReader::new(File::open(path.as_path())?);
        let exifreader = exif::Reader::new();
        let exif = exifreader
            .read_from_container(&mut bufreader)
            .with_context(|| format!("unable to read exif in {}", path.display()))?;
        let mut exif_datetime =
            match exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
                Some(field) => match &field.value {
                    Value::Ascii(s) => DateTime::from_ascii(s.first().with_context(|| {
                        format!("date field value missing in {}", path.display())
                    })?)?,
                    _ => bail!("invalid date field in {}", path.display()),
                },
                None => bail!("missing date field in {}", path.display()),
            };

        if let Some(field) = exif.get_field(Tag::OffsetTimeOriginal, In::PRIMARY) {
            if let Value::Ascii(s) = &field.value {
                // ignore any errors
                let _ = exif_datetime.parse_offset(s.first().unwrap());
            }
        }

        if let Some(field) = exif.get_field(Tag::SubSecTimeOriginal, In::PRIMARY) {
            if let Value::Ascii(s) = &field.value {
                // ignore any errors
                let _ = exif_datetime.parse_subsec(s.first().unwrap());
            }
        }

        let date = Date::from_calendar_date(
            exif_datetime.year.into(),
            exif_datetime.month.try_into()?,
            exif_datetime.day,
        )?;

        let time = Time::from_hms_nano(
            exif_datetime.hour,
            exif_datetime.minute,
            exif_datetime.second,
            exif_datetime.nanosecond.unwrap_or_default(),
        )?;

        // try to extract from image or locally or else assume UTC
        let offset = match exif_datetime.offset {
            Some(offset_minutes) => UtcOffset::from_whole_seconds(60i32 * offset_minutes as i32)?,
            None => UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC),
        };

        let datetime = PrimitiveDateTime::new(date, time).assume_offset(offset);

        Ok(Self { path, datetime })
    }
}

impl Ord for ImageFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.datetime.cmp(&other.datetime)
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
        self.datetime.hash(state);
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
        .chain(glob_with(&format!("{}/**/*.heic", base_dir), options)?)
        .collect::<Result<Vec<_>, _>>()?;

    let mut image_files = image_paths
        .into_iter()
        .map(|p| p.try_into())
        .collect::<Result<Vec<ImageFile>, _>>()?;

    image_files.sort();

    let years: HashSet<_> = image_files.iter().map(|imf| imf.datetime.year()).collect();

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
            (1u32, None::<OffsetDateTime>),
            |(prev_idx, prev_dt_option), imf| {
                let curr_dt = &imf.datetime;
                let mut curr_idx = 1;
                if let Some(prev_dt) = prev_dt_option {
                    if curr_dt.date() == prev_dt.date() {
                        curr_idx = *prev_idx + 1;
                    }
                }

                *prev_idx = curr_idx;
                *prev_dt_option = Some(*curr_dt);

                let new_path = tmp_dir
                    .join(curr_dt.format(format_description!("[year]")).unwrap())
                    .join(format!(
                        "{}_{:05}.{}",
                        curr_dt
                            .format(format_description!("[year]-[month]-[day]"))
                            .unwrap(),
                        curr_idx,
                        imf.path.extension().unwrap().to_str().unwrap().to_ascii_lowercase()
                    ));

                Some((imf, new_path))
            },
        )
        .collect::<HashMap<_, _>>();

    for (imf, new_path) in new_names_map {
        println!("{} -> {}", imf.path.display(), new_path.display());
        std::fs::set_permissions(&imf.path, Permissions::from_mode(0o644))?;
        filetime::set_file_mtime(&imf.path, SystemTime::from(imf.datetime).into())?;
        std::fs::rename(imf.path, &new_path).context("failed to do above rename")?;
    }

    Ok(())
}
