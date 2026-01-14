use anyhow::Result;
use log::info;
use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod ssh;
use ssh::session::Session;

#[derive(clap::Parser)]
pub struct Cli {
    #[clap(index = 1)]
    host: String,

    #[clap(long, short, default_value_t = 22)]
    port: u16,

    #[clap(long, short)]
    username: Option<String>,

    #[clap(long)]
    password: String,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    info!("Connecting to {}:{}", cli.host, cli.port);

    let mut ssh = Session::connect(
        cli.username.unwrap_or("root".to_string()),
        cli.password,
        (cli.host, cli.port),
    )
    .await?;
    info!("Connected");

    enable_raw_mode()?;
    ssh.run_shell().await?;
    ssh.close().await?;
    disable_raw_mode()?;
    Ok(())
}