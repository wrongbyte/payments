use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::transaction::{Dispute, Transaction, TransactionId, TransactionKind};

/// Client's assets account.
#[derive(Eq, PartialEq)]
pub struct Account {
    pub available: Decimal,
    pub held: Decimal,
    pub locked: bool,
    pub transactions: HashMap<TransactionId, Transaction>,
}

impl Account {
    pub fn new(initial_deposit: Decimal) -> Self {
        Self {
            available: initial_deposit,
            held: Decimal::ZERO,
            locked: false,
            transactions: HashMap::new(),
        }
    }

    /// Updates the client account accordingly to the new transaction received.
    pub fn process_transaction(
        &mut self,
        transaction: Transaction,
        disputes: &mut HashMap<TransactionId, Dispute>,
    ) {
        if self.locked {
            return;
        }

        let transaction_kind = transaction.kind;
        let tx_id = transaction.id;

        match transaction_kind {
            TransactionKind::Deposit { amount } => {
                if transaction.amount_is_valid() {
                    self.available += amount
                }
            }
            TransactionKind::Withdraw { amount } => {
                if transaction.amount_is_valid() && self.available > amount {
                    self.available -= amount
                }
            }
            TransactionKind::Dispute => {
                if disputes.contains_key(&tx_id) {
                    println!("This transaction already has an associated dispute.");
                }
                if let Some(transaction) = self.transactions.get(&tx_id)
                    && let Some(disputed_amount) = transaction.deposit_amount()
                {
                    let dispute = Dispute::new();
                    disputes.insert(tx_id, dispute);
                    self.hold_funds(disputed_amount);
                }
            }
            TransactionKind::Resolve => {
                if let Some(dispute) = disputes.get_mut(&tx_id)
                    && dispute.can_finish()
                    && let Some(disputed_amount) = self.disputed_deposit(tx_id)
                {
                    dispute.resolve();
                    self.release_held_funds(disputed_amount);
                }
            }
            TransactionKind::Chargeback => {
                if let Some(dispute) = disputes.get_mut(&tx_id)
                    && dispute.can_finish()
                    && let Some(disputed_amount) = self.disputed_deposit(tx_id)
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
