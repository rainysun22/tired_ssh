use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use log::info;
use russh::keys::*;
use russh::*;
use tokio::net::ToSocketAddrs;

use crossterm::terminal::{self, enable_raw_mode, disable_raw_mode};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

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
    ssh.start_shell().await?;
    ssh.close().await?;
    disable_raw_mode()?;
    Ok(())
}

struct Client {}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct Session {
    session: client::Handle<Client>,
}

impl Session {
    async fn connect<A: ToSocketAddrs>(
        user: impl Into<String>,
        password: impl Into<String>,
        addrs: A,
    ) -> Result<Self> {

        let config = client::Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            preferred: Preferred {
                kex: Cow::Owned(vec![
                    russh::kex::CURVE25519_PRE_RFC_8731,
                    russh::kex::EXTENSION_SUPPORT_AS_CLIENT,
                ]),
                ..Default::default()
            },
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut session = client::connect(config, addrs, sh).await?;
        let auth_res = session
            .authenticate_password(user, password)
            .await?;
        if !auth_res.success() {
            anyhow::bail!("Authentication (with password) failed");
        }

        Ok(Self { session })
    }

    async fn start_shell(&mut self) -> Result<()> {
        let mut channel = self.session.channel_open_session().await?;
        let (w, h) = terminal::size()?;
        channel.request_pty(false, "xterm", w as u32, h as u32, 0, 0, &[]).await?;
        channel.request_shell(false).await?;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

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