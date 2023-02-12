pub struct Customer {
    id: u64,
    name: String,
}

impl Reservation {
    fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }
}
