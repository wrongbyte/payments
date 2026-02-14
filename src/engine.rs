use rust_decimal::Decimal;

use crate::transaction::ClientId;

pub struct EngineOutput {
    pub client: ClientId,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}
