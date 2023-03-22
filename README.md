# exif renamer

this program
- parses the EXIF tag `DateTimeOriginal` (and `OffsetTimeOriginal` and `SubSecTimeOriginal` if present) from all `.jpg`, `.png` and `.heic` image files
  in the directory given as command line argument
- moves them to the directory `organizedRANDOMGIBBERISH/year`
- renames them to `YYYY-MM-DD_XXXXX.ext` where `XXXXX` is a zero-padded number starting from 1, sorted according to the extracted time
- changes file permissions to `644`
- changes last modified date to the extracted datetime

# build

```console
❯ make build
```

# install

```console
❯ make install
```

to install in a different path than `~/.local/bin`, set `PREFIX` before executing

# usage

```console
❯ exifrenamer BASE_DIRECTORY
```

where `BASE_DIRECTORY` is the directory containing image files

# example

```console
❯ ls -ltr ~/tmp
total 131156
-rwxr----- 1 a a 13759109 Jan 21 12:16 20230108_165449.jpg
-rwxr----- 1 a a 12595690 Jan 21 12:16 20230108_165512.jpg
-rwxr----- 1 a a 11484124 Jan 21 12:16 20230108_165604.jpg
-rwxr----- 1 a a 14372063 Jan 21 12:16 20230108_165744.jpg
-rwxr----- 1 a a 11216042 Jan 21 12:16 20230110_082132.jpg
-rwxr----- 1 a a 11071533 Jan 21 12:16 20230111_204336.jpg
-rwxr----- 1 a a 10972584 Jan 21 12:16 20230114_172036.jpg
-rwxr----- 1 a a 16491792 Jan 21 12:16 20230118_085020.jpg
-rwxr----- 1 a a 11806238 Jan 21 12:16 20230118_171824.jpg
-rwxr----- 1 a a  9900924 Jan 21 12:16 20230120_072249.jpg
-rwxr----- 1 a a 10604954 Jan 21 12:20 20221230_195726.jpg

❯ exifrenamer ~/tmp
/home/a/tmp/2023-01-11_00001.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-11_00001.jpg
/home/a/tmp/2023-01-10_00001.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-10_00001.jpg
/home/a/tmp/2023-01-14_00001.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-14_00001.jpg
/home/a/tmp/2023-01-18_00002.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-18_00002.jpg
/home/a/tmp/2023-01-08_00001.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-08_00001.jpg
/home/a/tmp/2023-01-20_00001.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-20_00001.jpg
/home/a/tmp/2023-01-18_00001.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-18_00001.jpg
/home/a/tmp/2023-01-08_00003.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-08_00003.jpg
/home/a/tmp/2023-01-08_00002.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-08_00002.jpg
/home/a/tmp/2022-12-30_00001.jpg -> /home/a/tmp/organizedXdiIVz/2022/2022-12-30_00001.jpg
/home/a/tmp/2023-01-08_00004.jpg -> /home/a/tmp/organizedXdiIVz/2023/2023-01-08_00004.jpg

❯ ls -lR ~/tmp   # updated modified time and permission
/home/a/tmp/:
total 4
drwxr-xr-x 4 a a 4096 Jan 21 12:36 organizedXdiIVz

/home/a/tmp/organizedXdiIVz:
total 8
drwxr-xr-x 2 a a 4096 Jan 21 12:36 2022
drwxr-xr-x 2 a a 4096 Jan 21 12:36 2023

/home/a/tmp/organizedXdiIVz/2022:
total 10360
-rw-r--r-- 1 a a 10604954 Dec 30 19:57 2022-12-30_00001.jpg

/home/a/tmp/organizedXdiIVz/2023:
total 120796
-rw-r--r-- 1 a a 13759109 Jan  8 16:54 2023-01-08_00001.jpg
-rw-r--r-- 1 a a 12595690 Jan  8 16:55 2023-01-08_00002.jpg
-rw-r--r-- 1 a a 11484124 Jan  8 16:56 2023-01-08_00003.jpg
-rw-r--r-- 1 a a 14372063 Jan  8 16:57 2023-01-08_00004.jpg
-rw-r--r-- 1 a a 11216042 Jan 10 08:21 2023-01-10_00001.jpg
-rw-r--r-- 1 a a 11071533 Jan 11 20:43 2023-01-11_00001.jpg
-rw-r--r-- 1 a a 10972584 Jan 14 17:20 2023-01-14_00001.jpg
-rw-r--r-- 1 a a 16491792 Jan 18 08:50 2023-01-18_00001.jpg
-rw-r--r-- 1 a a 11806238 Jan 18 17:18 2023-01-18_00002.jpg
-rw-r--r-- 1 a a  9900924 Jan 20 07:22 2023-01-20_00001.jpg
```
