use monzo_cli::{Config, Args, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::init()?;
    let args = Args::from_args();
    Cli::new_stdout(args, config).run().await?;
    Ok(())
}
