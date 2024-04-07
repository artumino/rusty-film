use chrono::NaiveDateTime;
use core::hash::Hasher;
use rexiv2::Metadata;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

extern crate rexiv2;

#[derive(Debug)]
pub struct Image<'a> {
    filename: &'a Path,
    out_file: Option<PathBuf>,
    original_date: Option<NaiveDateTime>,
    hash: u32,
    _exif: Option<Metadata>,
}

impl<'a> Image<'a> {
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn load(filename: &'a Path, destination: &Path) -> anyhow::Result<Image<'a>> {
        let file = File::open(filename)?;
        let exif = Metadata::new_from_path(filename).ok();
        let mut hash_reader = BufReader::new(&file);
        let hash = Image::compute_chunked_hash::<4096, _>(&mut hash_reader)?;
        let date = Image::get_exif_date(&exif);
        let output_path = Image::compute_output_path(filename, &date, hash, destination);
        Ok(Image {
            filename,
            hash,
            _exif: exif,
            original_date: date,
            out_file: output_path,
        })
    }

    fn compute_chunked_hash<const S: usize, R: Read>(reader: &mut R) -> anyhow::Result<u32> {
        #[cfg(feature = "tracing")]
        let span = tracing::span!(tracing::Level::INFO, "compute_chunked_hash");
        #[cfg(feature = "tracing")]
        let _guard = span.enter();
        let mut hasher = crc32c::Crc32cHasher::new(0); //crc32fast::Hasher::new();
        let mut buffer = [0; S];
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.write(&buffer[..bytes_read]);
        }
        Ok(hasher.finish() as u32)
    }

    pub fn filename(&self) -> &Path {
        self.filename
    }

    pub fn output_path(&self) -> Option<&PathBuf> {
        self.out_file.as_ref()
    }

    pub fn original_date(&self) -> Option<NaiveDateTime> {
        self.original_date
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn alread_exists(&self) -> bool {
        self.out_file
            .as_ref()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    pub fn hash(&self) -> u32 {
        self.hash
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn get_exif_date(exif: &Option<Metadata>) -> Option<NaiveDateTime> {
        let exif = exif.as_ref()?;
        let date = exif.get_tag_string("Exif.Image.DateTime").ok()?;
        let date = NaiveDateTime::parse_from_str(&date, "%Y:%m:%d %H:%M:%S").ok()?;
        Some(date)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    fn compute_output_path(
        filename: &Path,
        date: &Option<NaiveDateTime>,
        hash: u32,
        destination: &Path,
    ) -> Option<PathBuf> {
        let date = match date {
            Some(d) => *d,
            None => {
                use chrono::{offset::Utc, DateTime};
                let metadata = std::fs::metadata(filename).ok()?;
                let date = metadata.created().ok()?;
                let chrono: DateTime<Utc> = date.into();
                chrono.naive_utc()
            }
        };
        let extension = filename.extension()?;
        let extension = extension.to_str()?;
        let output_path = destination.join(format!(
            "{}_{:08X}.{}",
            date.format("%Y/%m/%d/%Y%m%d_%H%M%S"),
            hash,
            extension
        ));
        Some(output_path)
    }
}
