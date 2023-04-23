use std::ops::Range;

use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, Error, From, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

use super::{CustomerId, Price, ProstituteId};

/// 予約リポジトリ
#[async_trait::async_trait]
pub trait ReservationRepository {
    /// IDで予約を検索する
    async fn find_by_id(&self, id: ReservationId) -> Result<Option<Reservation>, DataAccessError>;
    /// 予約を保存する
    async fn save(&mut self, entity: &mut Reservation) -> Result<bool, DataAccessError>;
    /// 予約を削除する
    async fn delete(&mut self, entity: &mut Reservation) -> Result<bool, DataAccessError>;
}

/// 予約ID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ReservationId(u64);

impl Id for ReservationId {
    type Inner = u64;
}

/// 予約イベント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationEvent {
    /// 予約が作成された
    ReservationCreated {
        id: ReservationId,
        prostitute_ids: Vec<ProstituteId>,
        time: Range<DateTime<Utc>>,
        customer: ReservationCustomer,
    },
    /// 予約の詳細が追加された
    ReservationDetailAdded {
        id: ReservationId,
        detail: ReservationDetail,
    },
    /// 予約の詳細が削除された
    ReservationDetailDeleted {
        id: ReservationId,
        detail_id: ReservationDetailId,
    },
    /// 予約が削除された
    ReservationDeleted { id: ReservationId },
}

impl Event for ReservationEvent {
    type Id = ReservationId;
}

/// 予約エンティティ
#[derive(Debug, Default, Clone, IntoIterator, Serialize, Deserialize)]
pub struct Reservation {
    id: ReservationId,
    prostitute_ids: Vec<ProstituteId>,
    time: Range<DateTime<Utc>>,
    customer: ReservationCustomer,
    details: Vec<ReservationDetail>,
    #[serde(skip)]
    #[into_iterator]
    events: EventQueue<ReservationEvent>,
}

impl Reservation {
    pub fn create(
        id: ReservationId,
        prostitute_ids: Vec<ProstituteId>,
        time: Range<DateTime<Utc>>,
        customer: ReservationCustomer,
    ) -> Result<Self, ReservationError> {
        Self::validate_created(&prostitute_ids, &time, &customer)?;
        let mut entity = Reservation {
            id,
            prostitute_ids: prostitute_ids.clone(),
            time: time.clone(),
            customer: customer.clone(),
            ..Reservation::default()
        };
        entity.events.push(ReservationEvent::ReservationCreated {
            id,
            prostitute_ids,
            time,
            customer,
        });
        Ok(entity)
    }

    pub fn add_detail(&mut self, detail: ReservationDetail) -> Result<(), ReservationError> {
        self.validate_detail_added(&detail)?;
        self.details.push(detail.clone());
        self.events.push(ReservationEvent::ReservationDetailAdded {
            id: self.id,
            detail,
        });
        Ok(())
    }

    pub fn delete_detail(
        &mut self,
        detail_id: ReservationDetailId,
    ) -> Result<(), ReservationError> {
        self.validate_detail_deleted(&detail_id)?;
        self.details.retain(|d| d.id() != detail_id);
        self.events
            .push(ReservationEvent::ReservationDetailDeleted {
                id: self.id,
                detail_id,
            });
        Ok(())
    }

    pub fn prostitute_ids(&self) -> &[ProstituteId] {
        &self.prostitute_ids
    }

    pub fn time(&self) -> &Range<DateTime<Utc>> {
        &self.time
    }

    pub fn customer(&self) -> &ReservationCustomer {
        &self.customer
    }

    pub fn details(&self) -> &[ReservationDetail] {
        &self.details
    }

    fn validate_id(&self, id: &ReservationId) -> Result<(), ReservationError> {
        if self.id != *id {
            return Err(ReservationError::MismatchedId);
        }
        Ok(())
    }

    fn validate_created(
        prostitute_ids: &[ProstituteId],
        time: &Range<DateTime<Utc>>,
        customer: &ReservationCustomer,
    ) -> Result<(), ReservationError> {
        Self::validate_prostitute_ids(prostitute_ids)?;
        Self::validate_time(time)?;
        Self::validate_customer(customer)?;
        Ok(())
    }

    fn validate_detail_added(&self, detail: &ReservationDetail) -> Result<(), ReservationError> {
        if self.details.iter().any(|d| d.id == detail.id) {
            return Err(ReservationError::DuplicateDetail);
        }
        Ok(())
    }

    fn validate_detail_deleted(
        &self,
        detail_id: &ReservationDetailId,
    ) -> Result<(), ReservationError> {
        if !self.details.iter().any(|d| d.id == *detail_id) {
            return Err(ReservationError::DetailNotFound);
        }
        Ok(())
    }

    fn validate_prostitute_ids(prostitute_ids: &[ProstituteId]) -> Result<(), ReservationError> {
        if prostitute_ids.is_empty() {
            return Err(ReservationError::NoProstitutes);
        }
        Ok(())
    }

    fn validate_time(time: &Range<DateTime<Utc>>) -> Result<(), ReservationError> {
        if time.start >= time.end {
            return Err(ReservationError::InvalidTime);
        }
        Ok(())
    }

    fn validate_customer(customer: &ReservationCustomer) -> Result<(), ReservationError> {
        match customer {
            ReservationCustomer::Anonymous => Err(ReservationError::AnonymousNotAllowed),
            ReservationCustomer::Registered { .. } => Ok(()),
            ReservationCustomer::Unregistered { name, phone } => {
                if name.is_empty() {
                    return Err(ReservationError::UnregisteredCustomerNameRequired);
                }
                if phone.is_empty() {
                    return Err(ReservationError::UnregisteredCustomerPhoneRequired);
                }
                Ok(())
            }
        }
    }
}

impl Entity for Reservation {
    type Id = ReservationId;

    const ENTITY_NAME: &'static str = "reservation";

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Aggregation for Reservation {
    type Event = ReservationEvent;
    type Error = ReservationError;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            ReservationEvent::ReservationCreated {
                prostitute_ids,
                time,
                customer,
                ..
            } => {
                Self::validate_created(prostitute_ids, time, customer)?;
            }
            ReservationEvent::ReservationDetailAdded { id, detail } => {
                self.validate_id(id)?;
                self.validate_detail_added(detail)?;
            }
            ReservationEvent::ReservationDetailDeleted { id, detail_id } => {
                self.validate_id(id)?;
                self.validate_detail_deleted(detail_id)?;
            }
            ReservationEvent::ReservationDeleted { id } => {
                self.validate_id(id)?;
            }
        }
        Ok(())
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ReservationEvent::ReservationCreated {
                id,
                prostitute_ids,
                time,
                customer,
            } => {
                if self.id != id {
                    if let Ok(ebtity) = Self::create(id, prostitute_ids, time, customer) {
                        *self = ebtity;
                    }
                }
            }
            ReservationEvent::ReservationDetailAdded { id, detail } => {
                if self.id == id {
                    if let Err(_) = self.add_detail(detail) {};
                }
            }
            ReservationEvent::ReservationDetailDeleted { id, detail_id } => {
                if self.id == id {
                    if let Err(_) = self.delete_detail(detail_id) {};
                }
            }
            ReservationEvent::ReservationDeleted { .. } => {}
        }
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl PartialEq for Reservation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.prostitute_ids == other.prostitute_ids
            && self.time == other.time
            && self.customer == other.customer
            && self.details == other.details
    }
}

impl Eq for Reservation {}

/// 予約エラー
#[derive(Error, Display, Debug)]
pub enum ReservationError {
    /// IDが一致しません
    #[display(fmt = "ID does not match")]
    MismatchedId,
    /// 女の子が指定されていません
    #[display(fmt = "No prostitute_ids are specified")]
    NoProstitutes,
    /// 時間が不正です
    #[display(fmt = "Invalid time")]
    InvalidTime,
    /// 匿名の予約はできません
    #[display(fmt = "Anonymous reservation is not allowed")]
    AnonymousNotAllowed,
    /// 未登録のお客様の名前が指定されていません
    #[display(fmt = "Unregistered customer name is not specified")]
    UnregisteredCustomerNameRequired,
    /// 未登録のお客様の電話番号が指定されていません
    #[display(fmt = "Unregistered customer phone is not specified")]
    UnregisteredCustomerPhoneRequired,
    /// 予約詳細が重複しています
    #[display(fmt = "Duplicate reservation detail")]
    DuplicateDetail,
    /// 予約詳細が見つかりません
    #[display(fmt = "Reservation detail not found")]
    DetailNotFound,
    /// 予約詳細のエラー
    #[display(fmt = "Reservation detail error: {}", _0)]
    ReservationDetailError(#[error(source)] ReservationDetailError),
}

/// 予約したお客様
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ReservationCustomer {
    /// 匿名
    #[default]
    Anonymous,
    /// 登録済み
    Registered { id: CustomerId },
    /// 未登録
    Unregistered { name: String, phone: String },
}

/// 予約詳細ID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ReservationDetailId(u64);

impl Id for ReservationDetailId {
    type Inner = u64;
}

/// 予約詳細エンティティ
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReservationDetail {
    id: ReservationDetailId,
    name: String,
    quantity: u32,
    price: Price,
}

impl ReservationDetail {
    pub fn create(
        id: ReservationDetailId,
        name: String,
        quantity: u32,
        price: Price,
    ) -> Result<Self, ReservationDetailError> {
        Self::validate_created(&name, quantity, &price)?;
        Ok(ReservationDetail {
            id,
            name,
            quantity,
            price,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn price(&self) -> &Price {
        &self.price
    }

    fn validate_created(
        name: &str,
        quantity: u32,
        _price: &Price,
    ) -> Result<(), ReservationDetailError> {
        Self::validate_name(name)?;
        Self::validate_quantity(quantity)?;
        Ok(())
    }

    fn validate_name(name: &str) -> Result<(), ReservationDetailError> {
        if name.is_empty() {
            return Err(ReservationDetailError::NameRequired);
        }
        Ok(())
    }

    fn validate_quantity(quantity: u32) -> Result<(), ReservationDetailError> {
        if quantity < 1 {
            return Err(ReservationDetailError::InvalidQuantity);
        }
        Ok(())
    }
}

impl Entity for ReservationDetail {
    type Id = ReservationDetailId;

    const ENTITY_NAME: &'static str = "reservation_detail";

    fn id(&self) -> Self::Id {
        self.id
    }
}

/// 予約詳細エラー
#[derive(Error, Display, Debug)]
pub enum ReservationDetailError {
    /// 名前が指定されていません
    #[display(fmt = "Name is not specified")]
    NameRequired,
    /// 数量が不正です
    #[display(fmt = "Invalid quantity")]
    InvalidQuantity,
}
