use crate::account::Account;
use anyhow::Result;
use serde::Serialize;
use std::io;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Serialize)]
pub struct Output {
    pub client: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

#[derive(Debug)]
pub enum Command {
    Data(Account),
    CloseConnection,
}

pub struct Writer {
    receiver: UnboundedReceiver<Command>,
}

impl Writer {
    pub fn build(receiver: UnboundedReceiver<Command>) -> Self {
        Self { receiver }
    }

    // Receive the results through an unbounded channel and write them to the stdout output
    pub async fn run(&mut self) -> Result<()> {
        let mut writer = csv::WriterBuilder::new().from_writer(io::stdout());
        while let Some(data) = self.receiver.recv().await {
            match data {
                Command::CloseConnection => {
                    writer.flush()?;
                }
                Command::Data(account) => {
                    let held_funds = account.held_funds().parse::<f64>()?;
                    let available_funds = account.available_funds().parse::<f64>()?;
                    let client_report = Output {
                        client: account.client_id(),
                        available: available_funds,
                        held: held_funds,
                        total: available_funds + held_funds,
                        locked: account.locked(),
                    };
                    writer.serialize(client_report)?;
                }
            }
        }
        Ok(())
    }
}
