use crate::transaction::Transaction;
use std::collections::HashMap;

#[derive(Debug, Clone, Eq)]
pub struct Account {
    pub client_id: u16,
    pub held_funds: u64,
    pub available_funds: u64,
    pub locked: bool,
    // tx: info
    pub transaction_history: HashMap<u32, Transaction>,
}

// Implementation of PartialEq for testing purposes, I am omitting the comparison of
// `transaction history` because I do not have time to do such verbose tests
impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.client_id == other.client_id
            && self.held_funds == other.held_funds
            && self.available_funds == other.available_funds
            && self.locked == other.locked
            && self.transaction_history == other.transaction_history
    }
}
/// IMPORTANT NOTE: these function return bool instead of a custom error message/type
/// just because lack of time. Ideally they will return a proper error type.
impl Account {
    /// I must cap the precision to 0.0001, because if we do f64.to_bits()
    /// it will be transmuted with all the decimal part, reducing considerably
    /// the maximum number which could use
    const PRECISION: f64 = 0.0001;
    /// The maximum which can have is 15 digits
    const MAX_VALUE: u64 = 1_000_000_000_000_000;

    pub fn build(client_id: u16) -> Self {
        Self {
            client_id,
            held_funds: 0,
            available_funds: 0,
            locked: false,
            transaction_history: Default::default(),
        }
    }

    pub fn add(dest: &mut u64, amount: f64) -> bool {
        // it must be rounded because f64 representation of 5.0002 would be 5.000099999
        // and it would induce a precision error
        let transmuted_amount = (amount / Self::PRECISION).round() as u64;
        // sanity check
        if transmuted_amount + *dest > Self::MAX_VALUE
            || transmuted_amount > Self::MAX_VALUE
            || amount < 0.0
        {
            false
        } else {
            *dest += transmuted_amount;
            true
        }
    }

    fn substract(dest: &mut u64, amount: f64) -> bool {
        // it must be rounded because f64 representation of 5.0002 would be 5.000099999
        // and it would induce a precision error
        let transmuted_amount = (amount / Self::PRECISION).round() as u64;
        // sanity check
        if *dest < transmuted_amount || amount < 0.0 {
            false
        } else {
            *dest -= transmuted_amount;
            true
        }
    }

    fn add_held_funds(&mut self, amount: f64) -> bool {
        Self::add(&mut self.held_funds, amount)
    }

    fn substract_held_funds(&mut self, amount: f64) -> bool {
        Self::substract(&mut self.held_funds, amount)
    }

    pub fn block_funds(&mut self, amount: f64) -> bool {
        self.substract_funds(amount) && self.add_held_funds(amount)
    }

    pub fn unblock_funds(&mut self, amount: f64) -> bool {
        if self.substract_held_funds(amount) {
           if self.add_funds(amount) {
               true
           } else {
               self.add_held_funds(amount);
               false
           }
        } else {
            false
        }
    }

    pub fn retire_blocked_funds(&mut self, amount: f64) -> bool {
        self.substract_held_funds(amount)
    }

    pub fn add_funds(&mut self, amount: f64) -> bool {
        Self::add(&mut self.available_funds, amount)
    }

    pub fn substract_funds(&mut self, amount: f64) -> bool {
        Self::substract(&mut self.available_funds, amount)
    }

    pub fn available_funds(&self) -> String {
        format!("{:.04}", self.available_funds as f64 * Self::PRECISION)
    }

    pub fn held_funds(&self) -> String {
        format!("{:.04}", self.held_funds as f64 * Self::PRECISION)
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn client_id(&self) -> u16 {
        self.client_id
    }
}

#[cfg(test)]
mod test {
    use crate::account::Account;

    #[test]
    fn test_available_funds() {
        let mut account = Account::build(0);
        // Adding
        assert_eq!(account.available_funds(), "0.0000");
        assert!(account.add_funds(10.0));
        assert_eq!(account.available_funds(), "10.0000");
        assert!(!account.add_funds(-10.0));
        assert!(account.add_funds(1000.0));
        assert_eq!(account.available_funds(), "1010.0000");
        assert!(!account.add_funds(Account::MAX_VALUE as f64 * Account::PRECISION));
        assert_eq!(account.available_funds(), "1010.0000");
        assert!(account.add_funds(234924.4343));
        assert_eq!(account.available_funds(), "235934.4343");
        assert!(account.add_funds(Account::PRECISION));
        assert_eq!(account.available_funds(), "235934.4344");
        // Subtracting
        assert!(!account.substract_funds(Account::MAX_VALUE as f64 * Account::PRECISION));
        assert_eq!(account.available_funds(), "235934.4344");
        assert!(account.substract_funds(234924.4343));
        assert_eq!(account.available_funds(), "1010.0001");
        assert!(!account.substract_funds(234924.4343));
        assert_eq!(account.available_funds(), "1010.0001");
        assert!(account.substract_funds(1010.0000));
        assert_eq!(account.available_funds(), "0.0001");
        assert!(account.substract_funds(0.0001));
        assert_eq!(account.available_funds(), "0.0000");
    }

    #[test]
    fn test_held_funds() {
        let mut account = Account::build(0);
        assert!(!account.block_funds(100.0));
        assert_eq!(account.available_funds(), "0.0000");
        assert!(account.add_funds(100.0));
        assert_eq!(account.available_funds(), "100.0000");
        assert!(account.block_funds(99.0001));
        assert_eq!(account.available_funds(), "0.9999");
        assert_eq!(account.held_funds(), "99.0001");
    }
}
