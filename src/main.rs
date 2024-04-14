use clap::{Parser, Subcommand};
use enum_dispatch::enum_dispatch;
use import::ImportArgs;

mod image;
mod import;
mod io;
pub mod metadata;

#[derive(Debug, Parser)]
#[clap(name = "rusty-film", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[enum_dispatch(CommandRunner)]
pub enum Command {
    Import(ImportArgs),
}

#[enum_dispatch]
pub trait CommandRunner {
    fn run(&self) -> anyhow::Result<()>;
}

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tracing")]
    {
        use tracing_subscriber::layer::SubscriberExt;

        tracing::subscriber::set_global_default(
            tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default()),
        )
        .expect("setup tracy layer");
    }
    let app = App::parse();
    app.command.run()?;
    Ok(())
}
