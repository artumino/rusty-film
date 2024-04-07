use super::CommandRunner;
use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[clap(short, long)]
    pub source: PathBuf,

    #[clap(short, long)]
    pub destination: PathBuf,

    #[clap(short, long, default_value = "true")]
    pub recursive: bool,
}

impl CommandRunner for ImportArgs {
    async fn run(&self) -> anyhow::Result<()> {
        //Add fibonacci sequence
        let mut a = 0;
        let mut b = 1;
        for _ in 0..10 {
            println!("{}", a);
            let tmp = a;
            a = b;
            b += tmp;
        }
        Ok(())
    }
}
