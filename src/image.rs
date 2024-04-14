use chrono::NaiveDateTime;
use core::hash::Hasher;
#[cfg(feature = "memmap2")]
use memmap2::{Advice, Mmap};
use rexiv2::Metadata;
use serde::{Deserialize, Serialize};
use std::fs::File;
#[cfg(not(feature = "memmap2"))]
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

extern crate rexiv2;

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageInfo {
    filename: PathBuf,
    original_date: Option<NaiveDateTime>,
    hash: u32,
}

impl ImageInfo {
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn load(filename: &Path) -> anyhow::Result<ImageInfo> {
        let file = File::open(filename)?;
        let exif = Metadata::new_from_path(filename).ok();
        let hash = {
            #[cfg(not(feature = "memmap2"))]
            {
                ImageInfo::compute_chunked_hash::<4096>(&file)?
            }
            #[cfg(feature = "memmap2")]
            {
                ImageInfo::compute_hash(&file)?
            }
        };
        let date = ImageInfo::get_exif_date(&exif);
        Ok(ImageInfo {
            filename: filename.to_path_buf(),
            hash,
            original_date: date,
        })
    }

    #[cfg(not(feature = "memmap2"))]
    fn compute_chunked_hash<const S: usize>(file: &File) -> anyhow::Result<u32> {
        let mut reader = BufReader::new(file);

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

    #[cfg(feature = "memmap2")]
    fn compute_hash(file: &File) -> anyhow::Result<u32> {
        #[cfg(feature = "tracing")]
        let span = tracing::span!(tracing::Level::INFO, "compute_hash");
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        let mmap = unsafe { Mmap::map(file)? };
        mmap.advise(Advice::Sequential)?;
        let mut hasher = crc32c::Crc32cHasher::new(0);
        hasher.write(&mmap);
        Ok(hasher.finish() as u32)
    }

    pub fn filename(&self) -> &Path {
        &self.filename
    }

    pub fn output_path(&self, destination: &Path) -> Option<PathBuf> {
        ImageInfo::compute_output_path(&self.filename, &self.original_date, self.hash)
            .as_ref()
            .map(|path| destination.join(path))
    }

    pub fn original_date(&self) -> Option<NaiveDateTime> {
        self.original_date
    }

    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn alread_exists(&self, destination: &Path) -> bool {
        ImageInfo::compute_output_path(&self.filename, &self.original_date, self.hash)
            .as_ref()
            .map(|path| destination.join(path).exists())
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
    ) -> Option<String> {
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
        let output_path = format!(
            "{}_{:08X}.{}",
            date.format("%Y/%m/%d/%Y%m%d_%H%M%S"),
            hash,
            extension
        );
        Some(output_path)
    }
}
