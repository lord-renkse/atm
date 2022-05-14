use crate::account::Account;
use crate::parser::{Operation, TypeOperation};
use crate::processor::{OperationStatus, Processor};
use crate::transaction::{Transaction, TransactionType};

impl Processor {
    fn execute_deposit(account: &mut Account, operation: Operation) -> OperationStatus {
        if account.transaction_history.contains_key(&operation.tx) {
            return OperationStatus::RepeatedTransaction;
        }
        if let Some(amount) = operation.amount {
            if account.add_funds(amount) {
                OperationStatus::Successful(Transaction {
                    client_id: account.client_id(),
                    amount,
                    type_transaction: TransactionType::Deposit,
                })
            } else {
                OperationStatus::Unknown
            }
        } else {
            OperationStatus::EmptyAmount
        }
    }

    fn execute_withdrawal(account: &mut Account, operation: Operation) -> OperationStatus {
        if account.transaction_history.contains_key(&operation.tx) {
            return OperationStatus::RepeatedTransaction;
        }
        if let Some(amount) = operation.amount {
            if account.substract_funds(amount) {
                OperationStatus::Successful(Transaction {
                    client_id: account.client_id(),
                    amount,
                    type_transaction: TransactionType::Withdrawal,
                })
            } else {
                OperationStatus::Unknown
            }
        } else {
            OperationStatus::EmptyAmount
        }
    }

    fn execute_dispute(account: &mut Account, operation: Operation) -> OperationStatus {
        if let Some(transaction) = account.clone().transaction_history.get(&operation.tx) {
            if transaction.type_transaction != TransactionType::Deposit
                || operation.amount.is_some()
            {
                return OperationStatus::DisputeError;
            }
            if !account.block_funds(transaction.amount) {
                return OperationStatus::Unknown;
            }
            OperationStatus::UpdateTransaction(
                operation.tx,
                Transaction {
                    client_id: transaction.client_id,
                    amount: transaction.amount,
                    type_transaction: TransactionType::Dispute,
                },
            )
        } else {
            OperationStatus::NonExistingTx
        }
    }

    fn execute_resolve(account: &mut Account, operation: Operation) -> OperationStatus {
        if let Some(transaction) = account.clone().transaction_history.get(&operation.tx) {
            if transaction.type_transaction != TransactionType::Dispute
                || operation.amount.is_some()
            {
                return OperationStatus::DisputeError;
            }
            if !account.unblock_funds(transaction.amount) {
                return OperationStatus::DisputeError;
            }
            OperationStatus::UpdateTransaction(
                operation.tx,
                Transaction {
                    client_id: transaction.client_id,
                    amount: transaction.amount,
                    type_transaction: TransactionType::Deposit, // change it back as a normal deposit
                },
            )
        } else {
            OperationStatus::NonExistingTx
        }
    }

    fn execute_chargeback(account: &mut Account, operation: Operation) -> OperationStatus {
        if let Some(transaction) = account.clone().transaction_history.get(&operation.tx) {
            if transaction.type_transaction != TransactionType::Dispute
                || operation.amount.is_some()
            {
                return OperationStatus::DisputeError;
            }
            if !account.retire_blocked_funds(transaction.amount) {
                return OperationStatus::DisputeError;
            }
            account.lock();
            OperationStatus::UpdateTransaction(
                operation.tx,
                Transaction {
                    client_id: transaction.client_id,
                    amount: transaction.amount,
                    type_transaction: TransactionType::ChargedBack, // change it back as a normal deposit
                },
            )
        } else {
            OperationStatus::NonExistingTx
        }
    }

    // Dispatcher function
    pub fn execute_operation(account: &mut Account, operation: Operation) -> OperationStatus {
        // only execute operations if the account is not locked
        if account.locked {
            return OperationStatus::AccountLocked;
        }
        match operation.type_operation {
            TypeOperation::deposit => Self::execute_deposit(account, operation),
            TypeOperation::withdrawal => Self::execute_withdrawal(account, operation),
            TypeOperation::dispute => Self::execute_dispute(account, operation),
            TypeOperation::resolve => Self::execute_resolve(account, operation),
            TypeOperation::chargeback => Self::execute_chargeback(account, operation),
        }
    }
}
