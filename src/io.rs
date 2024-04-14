pub mod utils {
    pub fn safe_copy(
        source: &std::path::Path,
        destination: &std::path::Path,
        dry_run: bool,
    ) -> anyhow::Result<()> {
        if dry_run {
            println!(
                "Would copy {} to {}",
                source.display(),
                destination.display()
            );
            return Ok(());
        }

        let _ = std::fs::create_dir_all(destination.parent().unwrap());

        let temp_extension = destination.extension().unwrap_or_default();
        let temp_destination =
            destination.with_extension(format!("{}.tmp", temp_extension.to_string_lossy()));
        std::fs::copy(source, &temp_destination)?;
        std::fs::rename(&temp_destination, destination)?;

        Ok(())
    }
}
