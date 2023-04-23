use std::{collections::HashMap, ops::Range};

use chrono::{Utc, DateTime};
use derive_more::{Display, From, Deref};
use serde::{Serialize, Deserialize};

use crate::domain::Id;

use super::Price;

/// サービスID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ServiceId(u64);

impl Id for ServiceId {
    type Inner = u64;
}

/// サービスエンティティ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    id: ServiceId,
    name: String,
    description: String,
    default_price: Price,
    extension_price: Option<Price>,
    price: HashMap<Range<DateTime<Utc>>, Price>,
}