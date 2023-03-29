use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Reservation {
    id: u64,
    customer_id: u64,
    prostitute_id: u64,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}

impl Reservation {
    fn new(
        id: u64,
        customer_id: u64,
        prostitute_id: u64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            customer_id,
            prostitute_id,
            start_time,
            end_time,
        }
    }
}
