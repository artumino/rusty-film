This is a simple tool that given a glob path imports all the photos and related xmp metadata files with the following file structure:
`<DESTINATION_DIR>/<EXIF_YEAR>/<EXIF_MONTH>/<EXIF_DAY>/<EXIF_DATE>_<EXIF_TIME>_<CRC32C>.<EXTENSION>`

It tries to avoid duplicates and unecessary copies.

An example command that organized all png,jpg,cr2,cr3,dng photos:
`rusty-film import -s '/run/media/artumino/Photos/20[1-5][1-5]/**/*.[CJPDcjpd][RPNrpn][23GEge]' -d '/run/media/artumino/Library'`

A flag `--dry-run` can be prefixed to imports to simulate a run without making any filesystem changes
