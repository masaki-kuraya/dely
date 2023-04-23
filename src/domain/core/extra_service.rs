use async_trait::async_trait;
use derive_more::{Deref, Display, Error, From, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

use super::Money;

/// オプションサービスリポジトリ
#[async_trait]
pub trait ExtraServiceRepository {
    /// オプションサービスをIDで検索する
    async fn find_by_id(&self, id: ExtraServiceId)
        -> Result<Option<ExtraService>, DataAccessError>;
    /// オプションサービスを保存する
    async fn save(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError>;
    /// オプションサービスを削除する
    async fn delete(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError>;
}

/// オプションサービスID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ExtraServiceId(u64);

impl Id for ExtraServiceId {
    type Inner = u64;
}

/// オプションサービスイベント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtraServiceEvent {
    /// オプションサービスが作成された
    ExtraServiceCreated {
        id: ExtraServiceId,
        name: String,
        description: String,
        price: Money,
    },
    /// オプションサービス名が変更された
    ExtraServiceNameChanged {
        id: ExtraServiceId,
        name: String,
    },
    /// オプションサービス説明が変更された
    ExtraServiceDescriptionChanged {
        id: ExtraServiceId,
        description: String,
    },
    /// オプションサービス料金が変更された
    ExtraServicePriceChanged {
        id: ExtraServiceId,
        price: Money,
    },
    /// オプションサービスが削除された
    ExtraServiceDeleted {
        id: ExtraServiceId,
    },
}

impl Event for ExtraServiceEvent {
    type Id = ExtraServiceId;
}

/// オプションサービスエンティティ
#[derive(Debug, Default, Clone, IntoIterator, Serialize, Deserialize)]
pub struct ExtraService {
    id: ExtraServiceId,
    name: String,
    description: String,
    price: Money,
    #[serde(skip)]
    #[into_iterator]
    events: EventQueue<ExtraServiceEvent>,
}

impl ExtraService {
    pub fn create(
        id: ExtraServiceId,
        name: String,
        description: String,
        price: Money,
    ) -> Result<Self, ExtraServiceError> {
        Self::validate_created(&name)?;
        let mut entity = ExtraService {
            id,
            name: name.clone(),
            description: description.clone(),
            price: price.clone(),
            ..Default::default()
        };
        entity.events.push(ExtraServiceEvent::ExtraServiceCreated {
            id,
            name,
            description,
            price,
        });
        Ok(entity)
    }

    pub fn change_name(&mut self, name: String) -> Result<(), ExtraServiceError> {
        Self::validate_name_changed(&name)?;
        self.name = name.clone();
        self.events
            .push(ExtraServiceEvent::ExtraServiceNameChanged { id: self.id, name });
        Ok(())
    }

    pub fn change_description(&mut self, description: String) {
        self.description = description.clone();
        self.events.push(ExtraServiceEvent::ExtraServiceDescriptionChanged {
            id: self.id,
            description,
        });
    }

    pub fn change_price(&mut self, price: Money) {
        self.price = price.clone();
        self.events
            .push(ExtraServiceEvent::ExtraServicePriceChanged { id: self.id, price });
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn price(&self) -> &Money {
        &self.price
    }

    fn validate_id(&self, id: &ExtraServiceId) -> Result<(), ExtraServiceError> {
        match self.id == *id {
            true => Ok(()),
            false => Err(ExtraServiceError::MismatchedId),
        }
    }

    fn validate_created(name: &str) -> Result<(), ExtraServiceError> {
        Self::validate_name_changed(name)
    }

    fn validate_name_changed(name: &str) -> Result<(), ExtraServiceError> {
        match name.trim().is_empty() {
            true => Err(ExtraServiceError::NameIsBlank),
            false => Ok(()),
        }
    }
}

impl Entity for ExtraService {
    type Id = ExtraServiceId;

    const ENTITY_NAME: &'static str = "extra_service";

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Aggregation for ExtraService {
    type Event = ExtraServiceEvent;
    type Error = ExtraServiceError;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            ExtraServiceEvent::ExtraServiceCreated { name, .. } => Self::validate_created(name),
            ExtraServiceEvent::ExtraServiceNameChanged { id, name, .. } => {
                self.validate_id(id)?;
                Self::validate_name_changed(name)
            }
            ExtraServiceEvent::ExtraServiceDescriptionChanged { id, .. }
            | ExtraServiceEvent::ExtraServicePriceChanged { id, .. }
            | ExtraServiceEvent::ExtraServiceDeleted { id, .. } => self.validate_id(id),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ExtraServiceEvent::ExtraServiceCreated {
                id,
                name,
                description,
                price,
            } => {
                if self.id != id {
                    if let Ok(entity) = Self::create(id, name, description, price) {
                        *self = entity;
                    }
                }
            }
            ExtraServiceEvent::ExtraServiceNameChanged { id, name, .. } => {
                if self.id == id {
                    if let Err(_e) = self.change_name(name) {}
                }
            }
            ExtraServiceEvent::ExtraServiceDescriptionChanged {
                id, description, ..
            } => {
                if self.id == id {
                    self.change_description(description);
                }
            }
            ExtraServiceEvent::ExtraServicePriceChanged { id, price, .. } => {
                if self.id == id {
                    self.change_price(price);
                }
            }
            ExtraServiceEvent::ExtraServiceDeleted { .. } => {}
        }
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl PartialEq for ExtraService {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.description == other.description
            && self.price == other.price
    }
}

impl Eq for ExtraService {}

/// オプションサービスエラー
#[derive(Error, Display, Debug)]
pub enum ExtraServiceError {
    /// IDが一致しません
    #[display(fmt = "ID does not match")]
    MismatchedId,
    /// 名前が空欄です                  
    #[display(fmt = "Name cannot be blank")]
    NameIsBlank,
}

#[cfg(test)]
mod tests {
    use crate::domain::core::{Money, Currency};

    use super::*;

    #[tokio::test]
    async fn test_extra_service_create() {
        let service = ExtraService::create(
            ExtraServiceId(30),
            "AF".to_owned(),
            "アナルセックスを指します。".to_owned(),
            Money::new(10000, Currency::JPY),
        )
        .unwrap();
        assert_eq!(service.id(), ExtraServiceId(30));
        assert_eq!(service.name(), "AF");
        assert_eq!(service.description(), "アナルセックスを指します。");
        assert_eq!(service.price(), &Money::new(10000, Currency::JPY));
    }

    #[test]
    fn test_price_display() {
        let price = Money::new(1000000, Currency::JPY);
        assert_eq!(format!("{}", price), "¥1,000,000");
    }
}
