use super::image::ImageInfo;
use super::CommandRunner;
use crate::io::utils::safe_copy;

use anyhow::Context;
use clap::Args;
use directories::BaseDirs;
use env_logger::Logger;
use glob::glob;
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::{error, info, warn};
use persy::{Config, Persy};
use std::path::{Path, PathBuf};

extern crate rexiv2;

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[clap(
        short,
        long,
        help = "Glob pattern for the source folder (e.g., /mnt/media/**/*.cr3)"
    )]
    pub source: String,

    #[clap(
        short,
        long,
        help = "Destination folder of the import (files will be stored in $DESTINATION/<EXIF-YEAR>/<EXIF-MONTH>/<EXIF-DAY>/<EXIF-DATE>_<EXIF-TIME>_<CRC32C>.<EXTENSION>)"
    )]
    pub destination: PathBuf,

    #[clap(
        long,
        help = "Runs a dry-run, listing all operations without making any filesystem changes (except for cache files)"
    )]
    pub dry_run: bool,

    #[clap(short, long, help = "Avoids the creation and usage of any cache files")]
    pub no_cache: bool,
}

impl CommandRunner for ImportArgs {
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    fn run(&self, logger: Logger) -> anyhow::Result<()> {
        rexiv2::initialize()?;
        info!(
            "Importing files from {} to {}",
            self.source,
            self.destination.display()
        );
        let file_count = glob(&self.source)?.count();
        info!("Processing {} files", file_count);

        let cache = match self.no_cache {
            true => {
                info!("Cache disabled");
                None
            }
            false => {
                let base_dirs =
                    BaseDirs::new().context("Cannot determine base directories for caches")?;
                let cache_dir = base_dirs.cache_dir().join("rusty-film.cache");
                info!("Using cache file {}", cache_dir.display());
                let _ = Persy::create(&cache_dir);
                Some(Persy::open(&cache_dir, Config::new())?)
            }
        };

        let multi = MultiProgress::new();
        LogWrapper::new(multi.clone(), logger).try_init()?;
        let progress = multi.add(indicatif::ProgressBar::new(file_count as u64));
        progress.set_style(indicatif::ProgressStyle::default_bar().template(
            "{wide_msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta})",
        )?);
        for entry in glob(&self.source)? {
            match entry {
                Ok(path) => {
                    progress.set_message(format!("Processing {}", &path.display()));
                    let image = get_or_load_image(&path, &cache)?;

                    let output_path = match image.output_path(&self.destination) {
                        Some(p) => p,
                        None => {
                            info!(
                                "Error: Could not determine output path for image {}",
                                image.filename().display()
                            );
                            continue;
                        }
                    };
                    match image.already_exists(&self.destination) {
                        true => info!(
                            "Image {} already exists, skipping image copy...",
                            output_path.display()
                        ),
                        false => {
                            info!(
                                "Saving image {} to {}",
                                image.filename().display(),
                                output_path.display()
                            );

                            safe_copy(image.filename(), &output_path, self.dry_run)?;
                        }
                    }

                    let metadata_files =
                        super::metadata::discover_from_image(&image, &self.destination);
                    for metadata_file in metadata_files {
                        match metadata_file.is_up_to_date() {
                            true => info!(
                                "Metadata file {} already up-to-date, skipping copy...",
                                metadata_file.output_path().display()
                            ),
                            false => {
                                let metadata_out_path = metadata_file.output_path();
                                info!(
                                    "Saving metadata file {} to {}",
                                    metadata_file.filename().display(),
                                    metadata_out_path.display()
                                );

                                safe_copy(
                                    metadata_file.filename(),
                                    metadata_out_path,
                                    self.dry_run,
                                )?;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
            progress.inc(1);
        }

        progress.finish_with_message("Done");
        Ok(())
    }
}

fn get_or_load_image(image_path: &Path, cache: &Option<Persy>) -> anyhow::Result<ImageInfo> {
    let cached_image = load_from_cache(cache, image_path);

    if let Some(load_image) = cached_image {
        return Ok(load_image);
    }

    let load_image = ImageInfo::load(image_path)?;

    if let Some(cache) = cache {
        let image_path_str = image_path.to_string_lossy();
        let mut tx = cache.begin()?;
        tx.create_segment(&image_path_str)?;
        tx.insert(&image_path_str, &bincode::serialize(&load_image)?)?;
        let prepared = tx.prepare()?;
        prepared.commit()?;
    }

    Ok(load_image)
}

fn load_from_cache(cache: &Option<Persy>, image_path: &Path) -> Option<ImageInfo> {
    match cache {
        Some(cache) => {
            let image_path_str = image_path.to_string_lossy();
            cache
                .scan(&image_path_str)
                .ok()?
                .next()
                .and_then(|(_, data)| deserialize_or_remove(&image_path_str, data, cache))
        }
        _ => None,
    }
}

fn deserialize_or_remove(key: &str, data: Vec<u8>, cache: &Persy) -> Option<ImageInfo> {
    match bincode::deserialize(&data) {
        Ok(image) => Some(image),
        Err(e) => {
            warn!("Error deserializing cache data: {}, probably need to upgrade entry, removing old one", e);
            let mut tx = cache.begin().unwrap();
            tx.drop_segment(key).unwrap();
            let prepare = tx.prepare().unwrap();
            prepare.commit().unwrap();
            None
        }
    }
}
