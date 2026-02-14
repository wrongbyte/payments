use std::collections::HashMap;

use crate::{
    account::Account,
    transaction::{ClientId, Dispute, Transaction, TransactionId, TransactionKind},
};

pub mod account;
pub mod engine;
pub mod transaction;

fn main() {
    println!("Hello, world!");
}

pub fn process_transactions(inputs: Vec<Transaction>) {
    let mut clients: HashMap<ClientId, Account> = HashMap::new();
    let mut disputes: HashMap<TransactionId, Dispute> = HashMap::new();

    for transaction in inputs {
        let client_id = transaction.client;

        if let Some(client) = clients.get_mut(&client_id) {
            client.process_transaction(transaction, &mut disputes);
        } else {
            if let TransactionKind::Deposit { amount } = transaction.kind {
                let new_account = Account::new(amount);
                clients.insert(client_id, new_account);
            }
        }
    }
}
