use std::collections::HashMap;

use crate::{
    account::Account,
    transaction::{ClientId, Transaction, TransactionKind},
};
use rust_decimal::Decimal;

pub mod account;
pub mod transaction;

fn format_decimal(value: Decimal) -> String {
    format!("{:.4}", value)
}

fn main() -> std::io::Result<()> {
    let file = std::env::args()
        .nth(1)
        .expect("Missing the filename as the first argument");

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&file)?;

    let mut clients: HashMap<ClientId, Account> = HashMap::new();

    for transaction in reader.deserialize() {
        let transaction: Transaction = transaction?;

        let client_id = transaction.client;

        if let Some(client) = clients.get_mut(&client_id) {
            client.process_transaction(transaction);
        } else {
            if let TransactionKind::Deposit { amount } = transaction.kind {
                let new_account = Account::new(amount);
                clients.insert(client_id, new_account);
            }
        }
    }

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    wtr.write_record(&["client", "available", "held", "total", "locked"])?;

    for (client_id, account) in clients {
        wtr.write_record(&[
            client_id.0.to_string(),
            format_decimal(account.available),
            format_decimal(account.held),
            format_decimal(account.total_funds()),
            account.locked.to_string(),
        ])?;
    }

    Ok(())
}

pub fn process_transactions(inputs: Vec<Transaction>) {
    let mut clients: HashMap<ClientId, Account> = HashMap::new();

    for transaction in inputs {
        let client_id = transaction.client;

        if let Some(client) = clients.get_mut(&client_id) {
            client.process_transaction(transaction);
        } else {
            if let TransactionKind::Deposit { amount } = transaction.kind {
                let new_account = Account::new(amount);
                clients.insert(client_id, new_account);
            }
        }
    }
}
