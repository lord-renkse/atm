use crate::parser;
use crate::parser::Operation;
use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub enum Command {
    Data(Operation),
    CloseConnection,
}

pub struct Reader {
    sender: UnboundedSender<Command>,
}

impl Reader {
    pub fn build(sender: UnboundedSender<Command>) -> Self {
        Self { sender }
    }

    // Parse the CSV and send the Operations through an unbounded channel to the processor task
    pub async fn run(&self) -> Result<()> {
        let operations = parser::parse()?;
        for operation in operations {
            self.sender.send(Command::Data(operation))?;
            // There should be here a random time sleep to "emulate" a real operation
            // I didn't write it not to make slow the automated CLI tests
            // tokio::sleep(..).await;
        }
        self.sender.send(Command::CloseConnection)?;
        Ok(())
    }
}
