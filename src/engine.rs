use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::{
    account::Account,
    transaction::{ClientId, Transaction, TransactionKind},
};

pub struct EngineOutput {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

/// Accounts data.
#[derive(Default)]
pub struct Engine {
    pub clients: HashMap<ClientId, Account>,
}

impl Engine {
    pub fn process_transaction(&mut self, transaction: Transaction) {
        let client_id = transaction.client;

        if let Some(client) = self.clients.get_mut(&client_id) {
            client.process_transaction(transaction);
        } else {
            if let TransactionKind::Deposit { amount } = transaction.kind {
                let new_account = Account::new(amount);
                self.clients.insert(client_id, new_account);
            }
        }
    }
}
