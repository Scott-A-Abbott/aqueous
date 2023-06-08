use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Account {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub balance: i64,
    pub opened_time: Option<OffsetDateTime>,
    pub closed_time: Option<OffsetDateTime>,
    pub sequence: i64,
}

impl Account {
    pub fn was_opened(&self) -> bool {
        self.opened_time.is_some()
    }

    pub fn is_closed(&self) -> bool {
        self.closed_time.is_some()
    }

    pub fn deposit(&mut self, amount: i64) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: i64) {
        self.balance -= amount;
    }

    pub fn has_sufficient_funds(&self, amount: i64) -> bool {
        self.balance >= amount
    }

    pub fn has_processed(&self, message_sequence: i64) -> bool {
        self.sequence >= message_sequence
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            customer_id: Uuid::nil(),
            balance: 0,
            opened_time: None,
            closed_time: None,
            sequence: -1,
        }
    }
}
