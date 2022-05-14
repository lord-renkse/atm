use crate::account::Account;
use crate::parser::{Operation, TypeOperation};
use crate::transaction::Transaction;
use crate::{reader, writer};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod executors;

// Different type of status than an operation can result to
pub enum OperationStatus {
    Successful(Transaction),
    UpdateTransaction(u32, Transaction),
    AccountLocked,
    RepeatedTransaction,
    EmptyAmount,
    NonExistingTx,
    DisputeError,
    Unknown, // gathers many type of statuses
}

pub struct Processor {
    receiver: UnboundedReceiver<reader::Command>,
    sender: UnboundedSender<writer::Command>,
    // client_id: Account
    // it represents a SQL database table, in a real scenario it would be a database access boxed trait
    database: HashMap<u16, Account>,
}

impl Processor {
    pub fn build(
        receiver: UnboundedReceiver<reader::Command>,
        sender: UnboundedSender<writer::Command>,
    ) -> Self {
        Self {
            receiver,
            sender,
            database: Default::default(),
        }
    }

    // Auxiliary function to process the corresponding Operation
    fn process_data(&mut self, operation: Operation) {
        let client_id = operation.client;
        let tx = operation.tx;
        let account = self
            .database
            .entry(client_id)
            .or_insert_with(|| Account::build(client_id));
        match Self::execute_operation(account, operation) {
            OperationStatus::Successful(new_transaction) => {
                account.transaction_history.insert(tx, new_transaction);
            }
            OperationStatus::UpdateTransaction(tx, transaction) => {
                *account
                    .transaction_history
                    .get_mut(&tx)
                    .expect("unexpected error") = transaction;
            }
            // all the errors are ignored because of lack of time, they should be
            // processed accordingly
            _ => {}
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // it reads all the messages received from the queue
        while let Some(operation) = self.receiver.recv().await {
            // If the received command contains an operation
            match operation {
                // If the received command closes the connection: report the data to print it out
                // This is not done ideally, it was simplified for the sake of the exercise
                // The best solution would be to have a command per client to see their balance
                // and an administrator program would request the account balance for each existing account,
                // or an administrator would have access to the database where is everything
                reader::Command::CloseConnection => {
                    for account in self.database.values() {
                        self.sender.send(writer::Command::Data(account.clone()))?;
                    }
                    self.sender.send(writer::Command::CloseConnection)?;
                    break;
                }
                reader::Command::Data(operation) => {
                    // it is only possible to create an account with a deposit
                    if !self.database.contains_key(&operation.client)
                        && operation.type_operation != TypeOperation::deposit
                    {
                        continue;
                    }
                    self.process_data(operation);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::account::Account;
    use crate::parser::{Operation, TypeOperation};
    use crate::processor::Processor;
    use crate::{reader, writer};
    use std::collections::HashMap;
    use tokio::sync::mpsc;

    async fn test_all(list_operations: Vec<Operation>, expected_results: HashMap<u16, Account>) {
        let (sender_operations, receiver_operations) = mpsc::unbounded_channel::<reader::Command>();
        let (sender_results, mut receiver_results) = mpsc::unbounded_channel::<writer::Command>();
        let mut processor = Processor::build(receiver_operations, sender_results);

        let start_receiver = tokio::spawn(async move {
            for operation in list_operations {
                assert!(sender_operations
                    .send(reader::Command::Data(operation))
                    .is_ok());
            }
            assert!(sender_operations
                .send(reader::Command::CloseConnection)
                .is_ok());
        });

        assert!(processor.run().await.is_ok());

        while let Some(data) = receiver_results.recv().await {
            match data {
                writer::Command::Data(data) => {
                    assert!(check_data(data, &expected_results));
                }
                _ => break,
            }
        }

        assert!(start_receiver.await.is_ok());
    }

    fn check_data(input: Account, expected_resuts: &HashMap<u16, Account>) -> bool {
        match expected_resuts.get(&input.client_id) {
            Some(result_account) => {
                if *result_account == input {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    #[tokio::test]
    async fn test_simple() {
        let (list_operations, expected_result) = prepare_simple_test();
        test_all(list_operations, expected_result).await;
    }

    #[tokio::test]
    async fn test_complex() {
        let (list_operations, expected_result) = prepare_complex_test();
        test_all(list_operations, expected_result).await;
    }

    fn prepare_simple_test() -> (Vec<Operation>, HashMap<u16, Account>) {
        let list_operations = vec![
            Operation {
                type_operation: TypeOperation::deposit,
                client: 1,
                tx: 1,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 3,
                tx: 0,
                amount: Some(2.000100),
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 1,
                tx: 2,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::withdrawal,
                client: 1,
                tx: 202,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 3,
                tx: 1,
                amount: Some(1.000100),
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 5,
                tx: 4,
                amount: Some(5.000100),
            },
            Operation {
                type_operation: TypeOperation::withdrawal,
                client: 10,
                tx: 15,
                amount: Some(5.000100),
            },
            Operation {
                type_operation: TypeOperation::withdrawal,
                client: 5,
                tx: 105,
                amount: Some(5.000200),
            },
            Operation {
                type_operation: TypeOperation::withdrawal,
                client: 5,
                tx: 105,
                amount: Some(5.00000),
            },
        ];

        let expected_results: HashMap<u16, Account> = HashMap::from([
            (
                1,
                Account {
                    client_id: 1,
                    held_funds: 0,
                    available_funds: 0,
                    locked: false,
                    transaction_history: Default::default(),
                },
            ),
            (
                3,
                Account {
                    client_id: 3,
                    held_funds: 0,
                    available_funds: 30002,
                    locked: false,
                    transaction_history: Default::default(),
                },
            ),
            (
                5,
                Account {
                    client_id: 5,
                    held_funds: 0,
                    available_funds: 1,
                    locked: false,
                    transaction_history: Default::default(),
                },
            ),
        ]);

        (list_operations, expected_results)
    }

    fn prepare_complex_test() -> (Vec<Operation>, HashMap<u16, Account>) {
        let list_operations = vec![
            Operation {
                type_operation: TypeOperation::deposit,
                client: 1,
                tx: 0,
                amount: Some(502.000100),
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 1,
                tx: 2,
                amount: Some(320.000100),
            },
            Operation {
                type_operation: TypeOperation::dispute,
                client: 1,
                tx: 2,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::dispute,
                client: 2,
                tx: 0,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::dispute,
                client: 1,
                tx: 3,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::resolve,
                client: 1,
                tx: 3,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 1,
                tx: 200,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::resolve,
                client: 1,
                tx: 2,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::resolve,
                client: 1,
                tx: 2,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 1,
                tx: 201,
                amount: Some(0.000100),
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 2,
                tx: 300,
                amount: Some(1000.0),
            },
            Operation {
                type_operation: TypeOperation::dispute,
                client: 2,
                tx: 300,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 2,
                tx: 301,
                amount: Some(1000.0),
            },
            Operation {
                type_operation: TypeOperation::chargeback,
                client: 2,
                tx: 300,
                amount: None,
            },
            Operation {
                type_operation: TypeOperation::deposit,
                client: 2,
                tx: 301,
                amount: Some(1000.0),
            },
        ];

        let expected_results: HashMap<u16, Account> = HashMap::from([
            (
                1,
                Account {
                    client_id: 1,
                    held_funds: 0,
                    available_funds: 8220004,
                    locked: false,
                    transaction_history: Default::default(),
                },
            ),
            (
                2,
                Account {
                    client_id: 2,
                    held_funds: 0,
                    available_funds: 10000000,
                    locked: true,
                    transaction_history: Default::default(),
                },
            ),
        ]);

        (list_operations, expected_results)
    }
}
