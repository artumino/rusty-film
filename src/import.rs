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
}

impl CommandRunner for ImportArgs {
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
                    let image = Image::load(path.as_path())?;
                    progress.println(format!(
                        "Saving image {} to {}",
                        image.filename().display(),
                        image
                            .output_path(&self.destination)
                            .context("Invalid output path")?
                            .display()
                    ));
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
