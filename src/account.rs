use std::collections::HashMap;

use indexmap::IndexMap;
use rust_decimal::Decimal;

use crate::transaction::{Dispute, Transaction, TransactionId, TransactionKind};

/// The current state of a client's asset and transaction history.
#[derive(Eq, PartialEq)]
pub struct Account {
    /// Funds available for transactions
    pub available: Decimal,
    /// Funds held due to disputes.
    pub held: Decimal,
    /// If this account can do transactions
    pub locked: bool,
    /// History of transactions of this client, stored in
    /// chronological order.
    pub transactions: IndexMap<TransactionId, Transaction>,
    /// Disputes in this account.
    pub disputes: HashMap<TransactionId, Dispute>,
}

impl Account {
    pub fn new(initial_deposit: Decimal) -> Self {
        Self {
            available: initial_deposit,
            held: Decimal::ZERO,
            locked: false,
            transactions: IndexMap::new(),
            disputes: HashMap::new(),
        }
    }

    pub fn total_funds(&self) -> Decimal {
        self.available + self.held
    }

    /// Updates the client account accordingly to the new transaction received.
    pub fn process_transaction(&mut self, transaction: Transaction) {
        if self.locked {
            return;
        }

        let transaction_kind = transaction.kind;
        let tx_id = transaction.id;

        match transaction_kind {
            TransactionKind::Deposit { amount } => {
                if transaction.amount_is_valid() {
                    self.available += amount;
                    self.transactions.insert(tx_id, transaction);
                }
            }
            TransactionKind::Withdraw { amount } => {
                if transaction.amount_is_valid() && self.available > amount {
                    self.available -= amount;
                    self.transactions.insert(tx_id, transaction);
                }
            }
            TransactionKind::Dispute => {
                if self.disputes.contains_key(&tx_id) {
                    println!("This transaction already has an associated dispute.");
                }
                if let Some(transaction) = self.transactions.get(&tx_id)
                    && let Some(disputed_amount) = transaction.deposit_amount()
                {
                    let dispute = Dispute::new();
                    self.disputes.insert(tx_id, dispute);
                    self.hold_funds(disputed_amount);
                }
            }
            TransactionKind::Resolve => {
                let disputed_amount = self.disputed_deposit(tx_id);
                if let Some(dispute) = self.disputes.get_mut(&tx_id)
                    && dispute.can_finish()
                    && let Some(disputed_amount) = disputed_amount
                {
                    dispute.resolve();
                    self.release_held_funds(disputed_amount);
                }
            }
            TransactionKind::Chargeback => {
                let disputed_amount = self.disputed_deposit(tx_id);
                if let Some(dispute) = self.disputes.get_mut(&tx_id)
                    && dispute.can_finish()
                    && let Some(disputed_amount) = disputed_amount
                {
                    dispute.chargeback();
                    self.chargeback_and_lock(disputed_amount);
                }
            }
        }
    }

    /// Returns the disputed deposit transaction if it exists.
    pub fn disputed_deposit(&self, transaction_id: TransactionId) -> Option<Decimal> {
        let transaction = self.transactions.get(&transaction_id)?;
        transaction.deposit_amount()
    }

    /// Decreases the account's available funds and increases the `held` funds. Note that
    /// if the account does not have enough funds, this will result in a negative balance.
    /// However, since the held value increases by the same amount that available funds
    /// decrease, the total sum does not change.
    pub fn hold_funds(&mut self, disputed_amount: Decimal) {
        self.available -= disputed_amount;
        self.held += disputed_amount;
    }

    /// Releases the held funds back to the account available funds.
    pub fn release_held_funds(&mut self, disputed_amount: Decimal) {
        self.held -= disputed_amount;
        self.available += disputed_amount;
    }

    /// Withdraws the held funds from the account.
    pub fn chargeback_and_lock(&mut self, disputed_amount: Decimal) {
        self.held -= disputed_amount;
        self.available -= disputed_amount;
        self.locked = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::ClientId;

    use super::*;

    #[test]
    fn test_deposit() {
        let mut transactions = vec![];
        for i in 0..10 {
            transactions.push(Transaction {
                client: ClientId(1),
                kind: TransactionKind::Deposit {
                    amount: Decimal::new(10, 0),
                },
                id: TransactionId(i),
            });
        }
        let expected_available = Decimal::new(100, 0);
        let account = transactions
            .into_iter()
            .fold(Account::new(Decimal::ZERO), |mut acc, tx| {
                acc.process_transaction(tx);
                acc
            });

        assert_eq!(account.available, expected_available);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total_funds(), expected_available);
        for i in 0..10 {
            assert!(account.transactions.contains_key(&TransactionId(i)));
        }
    }

    #[test]
    fn test_withdraw() {
        let mut transactions = vec![];
        for i in 0..10 {
            transactions.push(Transaction {
                client: ClientId(1),
                kind: TransactionKind::Deposit {
                    amount: Decimal::new(10, 0),
                },
                id: TransactionId(i),
            });
        }

        transactions.push(Transaction {
            client: ClientId(1),
            kind: TransactionKind::Withdraw {
                amount: Decimal::new(5, 0),
            },
            id: TransactionId(15),
        });

        let expected_available = Decimal::new(95, 0);
        let account = transactions
            .into_iter()
            .fold(Account::new(Decimal::ZERO), |mut acc, tx| {
                acc.process_transaction(tx);
                acc
            });
        assert_eq!(account.available, expected_available);
        assert_eq!(account.held, Decimal::ZERO);
        assert!(account.transactions.contains_key(&TransactionId(15)));
        for i in 0..10 {
            assert!(account.transactions.contains_key(&TransactionId(i)));
        }
    }

    // Withdraw with insufficient funds should not be processed and should not be added to the transaction history.
    #[test]
    fn insufficient_funds() {
        let mut transactions = vec![];
        transactions.push(Transaction {
            client: ClientId(1),
            kind: TransactionKind::Deposit {
                amount: Decimal::new(5, 0),
            },
            id: TransactionId(1),
        });

        transactions.push(Transaction {
            client: ClientId(1),
            kind: TransactionKind::Withdraw {
                amount: Decimal::new(100, 0),
            },
            id: TransactionId(15),
        });

        let expected_available = Decimal::new(5, 0);
        let account = transactions
            .into_iter()
            .fold(Account::new(Decimal::ZERO), |mut acc, tx| {
                acc.process_transaction(tx);
                acc
            });
        assert_eq!(account.available, expected_available);
        assert_eq!(account.held, Decimal::ZERO);
        assert!(!account.transactions.contains_key(&TransactionId(15)));
    }

    // Basic dispute case
    #[test]
    fn test_dispute() {
        // Client deposits 100, then deposits 50 more, then disputes the first transaction.
        // 100 is moved to held, 50 is still available. Total is 150.
        let deposit_1 = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Deposit {
                amount: Decimal::new(100, 0),
            },
            id: TransactionId(1),
        };

        let deposit_2 = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Deposit {
                amount: Decimal::new(50, 0),
            },
            id: TransactionId(2),
        };

        let dispute = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Dispute,
            id: TransactionId(1), //first deposit
        };

        let transactions = vec![deposit_1, deposit_2, dispute];

        let account = transactions
            .into_iter()
            .fold(Account::new(Decimal::ZERO), |mut acc, tx| {
                acc.process_transaction(tx);
                acc
            });
        assert_eq!(account.available, Decimal::new(50, 0));
        assert_eq!(account.held, Decimal::new(100, 0));
        assert_eq!(account.total_funds(), Decimal::new(150, 0));
        assert_eq!(account.disputes.len(), 1);
        assert!(account.disputes.contains_key(&TransactionId(1)));
    }

    #[test]
    fn test_resolve() {
        // Client deposits 100, then disputes it, then resolves the dispute.
        // After the dispute, 100 is held and 0 is available. After the resolve, 100 is available again and 0 is held.
        let deposit = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Deposit {
                amount: Decimal::new(100, 0),
            },
            id: TransactionId(1),
        };

        let dispute = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Dispute,
            id: TransactionId(1), //first deposit
        };

        let resolve = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Resolve,
            id: TransactionId(1), //first deposit
        };

        let transactions = vec![deposit, dispute, resolve];

        let account = transactions
            .into_iter()
            .fold(Account::new(Decimal::ZERO), |mut acc, tx| {
                acc.process_transaction(tx);
                acc
            });
        assert_eq!(account.available, Decimal::new(100, 0));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total_funds(), Decimal::new(100, 0));
        assert_eq!(account.disputes.len(), 1);
        assert!(account.disputes.contains_key(&TransactionId(1)));
    }

    #[test]
    fn test_blocked_account_after_chargeback() {
        // Client deposits 100, then disputes it, then chargebacks the dispute.
        // After the dispute, 100 is held and 0 is available. After the chargeback, 0 is held, 0 is available and the account is locked.
        let deposit = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Deposit {
                amount: Decimal::new(100, 0),
            },
            id: TransactionId(1),
        };

        let dispute = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Dispute,
            id: TransactionId(1), //first deposit
        };
        // at this point client has 100 held and 0 available
        let chargeback = Transaction {
            client: ClientId(1),
            kind: TransactionKind::Chargeback,
            id: TransactionId(1), //first deposit
        };

        let transactions = vec![deposit, dispute, chargeback];

        let account = transactions
            .into_iter()
            .fold(Account::new(Decimal::ZERO), |mut acc, tx| {
                acc.process_transaction(tx);
                println!("After processing transaction {:?}, account state is: available: {}, held: {}, total: {}, locked: {}",
                    tx, acc.available, acc.held, acc.total_funds(), acc.locked);
                acc
            });
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total_funds(), Decimal::ZERO);
        assert_eq!(account.disputes.len(), 1);
        assert!(account.disputes.contains_key(&TransactionId(1)));
        assert!(account.locked);
    }
}
