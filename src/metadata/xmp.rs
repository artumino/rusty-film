use super::{ImageMetadataFile, IMAGE_METADATA_DISCOVERERS};
use crate::image::Image;
use glob::glob;
use linkme::distributed_slice;

#[distributed_slice(IMAGE_METADATA_DISCOVERERS)]
static XMP_METADATA_DISCOVERER: fn(image: &Image) -> Vec<ImageMetadataFile> =
    discover_metadata_files;

// This discovery method tries to find the most common XMP sidecar
// Some programs save these without the extension of the image, others like darktable include the
// extension
// Darktable also handles duplicate images as multiple xmp files with a suffix
#[cfg_attr(feature = "tracing", tracing::instrument)]
pub fn discover_metadata_files(image: &Image) -> Vec<ImageMetadataFile> {
    let filename = image.filename();
    let path = filename.parent().unwrap();
    let image_name = filename.file_stem().unwrap();
    glob(&format!(
        "{}/{}*.[Xx][Mm][Pp]",
        path.to_str().unwrap(),
        image_name.to_str().unwrap()
    ))
    .expect("Wrong glob pattern for xmp detector")
    .map(|entry| {
        let entry = entry.unwrap();
        let entry_extension = entry.extension().and_then(|x| x.to_str()).unwrap();
        let output_image_name = image
            .output_path()
            .and_then(|x| x.file_stem())
            .and_then(|x| x.to_str())
            .unwrap();
        let original_image_name = image
            .filename()
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap();
        let sidecar_filename = entry
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap()
            .replace(original_image_name, output_image_name);
        let output_path = image
            .output_path()
            .and_then(|x| x.parent())
            .unwrap()
            .join(sidecar_filename)
            .with_extension(entry_extension);
        let edit_date = std::fs::metadata(&entry)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .unwrap();
        let output_edit_date = std::fs::metadata(&output_path)
            .ok()
            .and_then(|metadata| metadata.modified().ok());
        ImageMetadataFile {
            filename: entry,
            output_path,
            output_edit_date,
            edit_date,
        }
    })
    .collect()
}
