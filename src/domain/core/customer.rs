use derive_more::{Deref, Display, From};
use serde::{Deserialize, Serialize};

use crate::domain::Id;

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct CustomerId(u64);

impl Id for CustomerId {
    type Inner = u64;
}

pub struct Customer {
    id: u64,
    name: String,
}

impl Customer {
    fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }
}
