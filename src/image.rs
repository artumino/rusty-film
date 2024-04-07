use chrono::NaiveDateTime;
use core::hash::Hasher;
use rexiv2::Metadata;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

extern crate rexiv2;

pub struct Image<'a> {
    filename: &'a Path,
    hash: u32,
    exif: Metadata,
    metadata_files: Vec<ImageMetadataFile>,
}

pub struct ImageMetadataFile {
    filename: PathBuf,
    includes_image_extensions: bool,
}

impl<'a> Image<'a> {
    pub fn load(filename: &Path) -> anyhow::Result<Image> {
        let file = File::open(filename)?;
        let exif = Metadata::new_from_path(filename)?;
        let mut hash_reader = BufReader::new(&file);
        let hash = Image::compute_chunked_hash::<4096, _>(&mut hash_reader)?;

        let metadata_files = vec![];
        Ok(Image {
            filename,
            hash,
            exif,
            metadata_files,
        })
    }

    fn compute_chunked_hash<const S: usize, R: Read>(reader: &mut R) -> anyhow::Result<u32> {
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

    pub fn hash(&self) -> u32 {
        self.hash
    }

    pub fn get_exif_date(&self) -> Option<NaiveDateTime> {
        let date = self.exif.get_tag_string("Exif.Image.DateTime").ok()?;
        let date = NaiveDateTime::parse_from_str(&date, "%Y:%m:%d %H:%M:%S").ok()?;
        Some(date)
    }

    pub fn output_path(&self, destination: &Path) -> Option<PathBuf> {
        let date = self.get_exif_date()?;
        let extension = self.filename.extension()?;
        let extension = extension.to_str()?;
        let output_path = destination.join(format!(
            "{}_{:08X}.{}",
            date.format("%Y/%m/%d/%Y%m%d_%H%M%S"),
            self.hash,
            extension
        ));
        Some(output_path)
    }
}
