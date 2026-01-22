use anyhow::Result;

use termwiz::caps::Capabilities;
use termwiz::terminal::Terminal as TermwizTerminal;
use termwiz::terminal::new_terminal;
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::Change;

use crate::app::ssh::session::Session;
use crate::app::ssh::session::SshEvent;

pub struct Terminal {
    termwiz: Box<dyn TermwizTerminal>,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let caps= Capabilities::new_from_env()?;
        let termwiz = Box::new(new_terminal(caps)?);
        Ok(Self { termwiz })
    }

    pub async fn input(&mut self, session: &mut Session) -> Result<()> {
        while let Some(input) = self.termwiz.poll_input(None)? {
            match input {
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    modifiers: _,
                }) => {
                    session.send(c.to_string().as_bytes()).await?;
                }
                _ => println!("{:?}", input),
            }
        }
        Ok(())
    }

    pub async fn render(&mut self, data: Vec<u8>) -> Result<()> {
        println!("{:?}", data);
        Ok(())
    }
}