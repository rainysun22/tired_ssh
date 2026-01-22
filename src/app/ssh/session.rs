use std::borrow::Cow;
use std::sync::Arc;
use anyhow::Result;
use log::info;
use russh::client::Msg;
use russh::*;
use russh::ChannelMsg;
use tokio::net::ToSocketAddrs;

use super::client::Client;

pub struct Session {
    session: client::Handle<Client>,
    channel: Channel<Msg>,
}

pub struct PtyOptions<'a> {
    pub term: &'a str,
    pub width: u32,
    pub height: u32,
}

pub enum SshEvent {
    Data(Vec<u8>),
    Exit(u32),
    Closed,
}

impl Session {
    pub async fn connect<A: ToSocketAddrs>(
        user: impl Into<String>,
        password: impl Into<String>,
        addrs: A,
    ) -> Result<client::Handle<Client>> {

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
        let client = Client {};

        let mut session = client::connect(config, addrs, client).await?;
        let auth_res = session
            .authenticate_password(user, password)
            .await?;
        if !auth_res.success() {
            anyhow::bail!("Authentication (with password) failed");
        }

        Ok(session)
    }

    pub async fn open(session: client::Handle<Client>) -> Result<Self> {
        let channel = session.channel_open_session().await?;
        Ok(Self { session: session, channel: channel })
    }

    pub async fn request_pty(&mut self, opts: PtyOptions<'_>) -> Result<()> {
        self.channel.request_pty(false, opts.term, opts.width, opts.height, 0, 0, &[]).await?;
        Ok(())
    }

    pub async fn start_shell(&mut self) -> Result<()> {
        self.channel.request_shell(false).await?;
        Ok(())
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<()> {
        self.channel.data(data).await?;
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<SshEvent> {
        match self.channel.wait().await {
            Some(ChannelMsg::Data { data }) => {
                Some(SshEvent::Data(data.to_vec()))
            }
            Some(ChannelMsg::ExtendedData { data, .. }) => {
                Some(SshEvent::Data(data.to_vec()))
            }
            Some(ChannelMsg::ExitStatus { exit_status }) => {
                info!("Exit status: {}", exit_status);
                Some(SshEvent::Exit(exit_status))
            }
            None => Some(SshEvent::Closed),
            _ => None,

        }
    }

    pub async fn close(&mut self) -> Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}
