use anyhow::Result;
use log::info;
use clap::Parser;

mod ssh;
use ssh::session::{Session, PtyOptions, SshEvent};
mod terminal;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    enable_raw_mode()?;
    ssh.request_pty(pty).await?;
    ssh.start_shell().await?;

    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut buf = [0; 1024];

    loop {
        tokio::select! {
            output = ssh.next_event() => {
                match output {
                    Some(SshEvent::Data(data)) => {
                        stdout.write_all(&data).await?;
                        stdout.flush().await?;
                    }
                    Some(SshEvent::Exit(_)) => break,
                    Some(SshEvent::Closed) => break,
                    None => {},
                }
            },
            input = stdin.read(&mut buf) => {
                match input {
                    Ok(0) => break,
                    Ok(n) => {
                        ssh.send(&buf[..n]).await?;
                    }
                    Err(e) => anyhow::bail!("Error reading from stdin: {}", e),
                }
            },
        }
    }
    ssh.close().await?;
    disable_raw_mode()?;
    Ok(())
}
