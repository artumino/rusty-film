use super::image::ImageInfo;
use linkme::distributed_slice;
use std::path::{Path, PathBuf};
pub mod xmp;

pub struct ImageMetadataFile {
    filename: PathBuf,
    output_path: PathBuf,
    output_edit_date: Option<std::time::SystemTime>,
    edit_date: std::time::SystemTime,
}

impl ImageMetadataFile {
    pub fn filename(&self) -> &PathBuf {
        &self.filename
    }

    pub fn output_path(&self) -> &PathBuf {
        &self.output_path
    }

    pub fn is_up_to_date(&self) -> bool {
        self.output_edit_date
            .map(|date| date >= self.edit_date)
            .unwrap_or(false)
    }
}

#[distributed_slice]
pub static IMAGE_METADATA_DISCOVERERS: [fn(
    image: &ImageInfo,
    destination: &Path,
) -> Vec<ImageMetadataFile>];

#[cfg_attr(feature = "tracing", tracing::instrument)]
pub fn discover_from_image(image: &ImageInfo, destination: &Path) -> Vec<ImageMetadataFile> {
    IMAGE_METADATA_DISCOVERERS
        .iter()
        .flat_map(|discoverer| discoverer(image, destination))
        .collect()
}
