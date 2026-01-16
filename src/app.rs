use anyhow::Result;
use log::info;
use clap::Parser;

mod ssh;
use ssh::session::{Session, PtyOptions, SshEvent};
mod terminal;

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

    let handle = Session::connect(
        cli.username.unwrap_or("root".to_string()),
        cli.password,
        (cli.host, cli.port),
    )
    .await?;
    info!("Connected");

    let mut ssh  = Session::open(handle).await?;
    let pty = PtyOptions {
        term: "xterm",
        width: 80,
        height: 24,
    };
    ssh.request_pty(pty).await?;
    ssh.start_shell().await?;
    ssh.close().await?;
    Ok(())
}