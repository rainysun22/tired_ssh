use std::borrow::Cow;
use std::sync::Arc;
use anyhow::Result;
use log::info;
use russh::client::Msg;
use russh::*;
use russh::ChannelMsg;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::ToSocketAddrs;

use crossterm::terminal;

use super::client::Client;

pub struct Session {
    session: client::Handle<Client>,
}

impl Session {
    pub async fn connect<A: ToSocketAddrs>(
        user: impl Into<String>,
        password: impl Into<String>,
        addrs: A,
    ) -> Result<Self> {

        let config = client::Config {
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

        Ok(Self { session: session })
    }

    pub async fn run_shell(&mut self) -> Result<()> {
        let mut channel = self.session.channel_open_session().await?;
        let (w, h) = terminal::size()?;
        channel.request_pty(false, "xterm", w as u32, h as u32, 0, 0, &[]).await?;
        channel.request_shell(false).await?;
        self.run(&mut channel).await?;
        Ok(())
    }

    async fn run(&self, channel: &mut Channel<Msg>) -> Result<()> {
        let mut stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut buf = [0; 1024];

        loop {
            tokio::select! {
                res = stdin.read(&mut buf) => {
                    match res {
                        Ok(0) => break,
                        Ok(n) => {
                            channel.data(&buf[..n]).await?;
                        }
                        Err(e) => anyhow::bail!("Error reading from stdin: {}", e),
                    }
                }
                msg = channel.wait() => {
                    match msg {
                        Some(ChannelMsg::Data { data }) => {
                            stdout.write_all(&data).await?;
                            stdout.flush().await?;
                        }
                        Some(ChannelMsg::ExtendedData { data, .. }) => {
                            stdout.write_all(&data).await?;
                            stdout.flush().await?;
                        }
                        Some(ChannelMsg::ExitStatus { exit_status }) => {
                            info!("Exit status: {}", exit_status);
                            break;
                        }
                        None => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}