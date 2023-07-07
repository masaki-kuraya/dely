use std::{collections::HashMap, ops::Range};

use chrono::{Date, DateTime, NaiveDate, NaiveTime, Utc};
use derive_more::{Deref, Display, Error, From};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, Entity, Id};

use super::{Money, Price};

/// サービスID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ServiceId(u64);

impl Id for ServiceId {
    type Inner = u64;
}

/// サービスイベント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceEvent {
    /// サービスが作成された
    ServiceCreated {
        id: ServiceId,
        name: String,
        description: String,
        default_price: Price,
        extension_price: Option<Price>,
        time_based_prices: HashMap<Range<NaiveTime>, Price>,
        discounts: Vec<Discount>,
    },
    /// サービス名が変更された
    ServiceNameChanged { id: ServiceId, name: String },
    /// サービス説明が変更された
    ServiceDescriptionChanged { id: ServiceId, description: String },
    /// サービスのデフォルト価格が変更された
    ServiceDefaultPriceChanged { id: ServiceId, default_price: Price },
    /// サービスの延長価格が変更された
    ServiceExtensionPriceChanged {
        id: ServiceId,
        extension_price: Option<Price>,
    },
    /// サービスの時間帯価格が変更された
    ServiceTimeBasedPriceChanged {
        id: ServiceId,
        time_based_prices: HashMap<Range<NaiveTime>, Price>,
    },
    /// サービスの時間帯価格が追加された
    ServiceTimeBasedPriceAdded {
        id: ServiceId,
        time: Range<NaiveTime>,
        price: Price,
    },
    /// サービスの時間帯価格が削除された
    ServiceTimeBasedPriceDeleted {
        id: ServiceId,
        time: Range<NaiveTime>,
    },
    /// サービスの割引が変更された
    ServiceDiscountChanged {
        id: ServiceId,
        discounts: Vec<Discount>,
    },
    /// サービスの割引が追加された
    ServiceDiscountAdded { id: ServiceId, discount: Discount },
    /// サービスの割引が削除された
    ServiceDiscountDeleted { id: ServiceId, discount: Discount },
    /// サービスが削除された
    ServiceDeleted { id: ServiceId },
}

/// サービスエンティティ
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Service {
    id: ServiceId,
    name: String,
    description: String,
    default_price: Price,
    extension_price: Option<Price>,
    time_based_prices: HashMap<Range<NaiveTime>, Price>,
    discounts: Vec<Discount>,
}

impl Service {
    pub fn calculate_price(&self, date_time: DateTime<Utc>) -> Money {
        todo!("calculate_price")
    }

    pub fn base_price(&self, date_time: DateTime<Utc>) -> Money {
        todo!("base_price")
    }

    pub fn discount_price(&self, date_time: DateTime<Utc>) -> Money {
        todo!("discount_price")
    }

    fn validate_id(&self, id: &ServiceId) -> Result<(), ServiceError> {
        if self.id != *id {
            return Err(ServiceError::MismatchedId);
        }
        Ok(())
    }

    fn validate_name(&self, name: &str) -> Result<(), ServiceError> {
        if name.trim().is_empty() {
            return Err(ServiceError::NameIsBlank);
        }
        Ok(())
    }

    fn validate_default_price(&self, price: &Price) -> Result<(), ServiceError> {
        self.validate_price_is_negative(price)
    }

    fn validate_price_is_negative(&self, price: &Price) -> Result<(), ServiceError> {
        if price.is_negative() {
            return Err(ServiceError::PriceIsNegative);
        }
        Ok(())
    }

}

impl Entity for Service {
    type Id = ServiceId;

    const ENTITY_NAME: &'static str = "service";

    fn id(&self) -> Self::Id {
        self.id
    }
}

/// サービスエラー
#[derive(Error, Display, Debug)]
pub enum ServiceError {
    /// IDが一致しません
    #[display(fmt = "ID does not match")]
    MismatchedId,
    /// 名前が空欄です                  
    #[display(fmt = "Name cannot be blank")]
    NameIsBlank,
    /// 価格が負です
    #[display(fmt = "Price cannot be negative")]
    PriceIsNegative,
}

/// 割引の種類
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscountType {
    /// 値引き額
    Amount { amount: Price },
    /// 割引率
    Percentage { percentage: i32 },
}

/// 割引
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discount {
    _type: DiscountType,
    period: Range<DateTime<Utc>>,
}
