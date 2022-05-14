#[derive(Debug, Clone, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    ChargedBack,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub client_id: u16,
    pub amount: f64,
    pub type_transaction: TransactionType,
}

// For testing purposes
impl Eq for Transaction {}
