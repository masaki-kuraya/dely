#[derive(Debug)]
pub struct Prostitute {
    id: u64,
    name: String,
}

impl Prostitute {
    fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }
}
