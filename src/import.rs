use super::image::Image;
use super::CommandRunner;
use anyhow::Context;
use clap::Args;
use glob::glob;
use std::path::PathBuf;

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
        short,
        long,
        help = "Runs a dry-run, listing all operations without making any filesystem changes"
    )]
    pub dry_run: bool,
}

impl CommandRunner for ImportArgs {
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    fn run(&self) -> anyhow::Result<()> {
        rexiv2::initialize()?;
        println!(
            "Importing files from {} to {}",
            self.source,
            self.destination.display()
        );
        let file_count = glob(&self.source)?.count();
        println!("Processing {} files", file_count);
        let progress = indicatif::ProgressBar::new(file_count as u64);
        progress.set_style(indicatif::ProgressStyle::default_bar().template(
            "{wide_msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta})",
        )?);
        for entry in glob(&self.source)? {
            match entry {
                Ok(path) => {
                    progress.set_message(format!("Processing {}", &path.display()));
                    let image = Image::load(path.as_path(), self.destination.as_path())?;

                    let output_path = image.output_path().context("Invalid output path")?;
                    match image.alread_exists() {
                        true => progress.println(format!(
                            "Image {} already exists, skipping image copy...",
                            output_path.display()
                        )),
                        false => {
                            progress.println(format!(
                                "Saving image {} to {}",
                                image.filename().display(),
                                output_path.display()
                            ));

                            if !self.dry_run {
                                let _ = std::fs::create_dir_all(output_path);
                                std::fs::copy(image.filename(), output_path)?;
                            }
                        }
                    }

                    let metadata_files = super::metadata::discover_from_image(&image);
                    for metadata_file in metadata_files {
                        match metadata_file.is_up_to_date() {
                            true => progress.println(format!(
                                "Metadata file {} already up-to-date, skipping copy...",
                                metadata_file.output_path().display()
                            )),
                            false => {
                                let metadata_out_path = metadata_file.output_path();
                                progress.println(format!(
                                    "Saving metadata file {} to {}",
                                    metadata_file.filename().display(),
                                    metadata_out_path.display()
                                ));
                                if !self.dry_run {
                                    let _ = std::fs::create_dir_all(metadata_out_path);
                                    std::fs::copy(metadata_file.filename(), metadata_out_path)?;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
            progress.inc(1);
        }

        progress.finish_with_message("Done");
        Ok(())
    }
}
