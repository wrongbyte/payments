use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClientId(u16);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TransactionId(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransactionKind {
    Deposit { amount: Decimal },
    Withdraw { amount: Decimal },
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Transaction {
    pub kind: TransactionKind,
    pub client: ClientId,
    pub id: TransactionId,
}

impl Transaction {
    /// If this transaction is part of a dispute process.
    pub fn belongs_to_dispute(&self) -> bool {
        let kind = self.kind;
        matches!(
            kind,
            TransactionKind::Dispute | TransactionKind::Resolve | TransactionKind::Chargeback
        )
    }

    /// Amount, if the operation is a deposit.
    pub fn deposit_amount(&self) -> Option<Decimal> {
        let TransactionKind::Deposit { amount } = self.kind else {
            return None;
        };
        Some(amount)
    }

    /// Checks if the transaction amount is a valid one.
    pub fn is_valid(&self) -> bool {
        match self.kind {
            TransactionKind::Deposit { amount } => amount > Decimal::ZERO,
            TransactionKind::Withdraw { amount } => amount > Decimal::ZERO,
            _ => true,
        }
    }
}

pub struct EngineOutput {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

#[derive(PartialEq, Eq)]
pub enum DisputeState {
    /// Initial state of a dispute.
    Disputed,
    /// The dispute was resolved and held funds were made available
    /// again for the client.
    Resolved,
    /// The dispute was finished with a chargeback, withdrawing
    /// from the client.
    ChargedBack,
}

/// A dispute is a claim that a previously processed transaction (specifically a deposit)
/// was erroneous or fraudulent and should be reversed.
/// A dispute references the original transaction by ID and can be followed by either a
/// resolve (releasing the held funds back to available) or a chargeback (removing the held
/// funds and freezing the account).
pub struct Dispute {
    state: DisputeState,
}

impl Dispute {
    pub fn new() -> Self {
        Self {
            state: DisputeState::Disputed,
        }
    }

    /// If we can finish the dispute, either via a resolve or a chargeback.
    pub fn can_finish(&self) -> bool {
        matches!(self.state, DisputeState::Disputed)
    }

    pub fn resolve(&mut self) {
        self.state = DisputeState::Resolved
    }

    pub fn chargeback(&mut self) {
        self.state = DisputeState::ChargedBack
    }
}
