use crate::domain::{DataAccessError, Entity, Event, EventQueue, EventQueueIntoIter, Id};
use async_trait::async_trait;
use core::fmt;
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use thiserror::Error;

#[async_trait]
pub trait ExtraServiceRepository {
    async fn find_by_id(&self, id: ExtraServiceId)
        -> Result<Option<ExtraService>, DataAccessError>;
    async fn save(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError>;
    async fn delete(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExtraServiceId(u64);

impl Id for ExtraServiceId {
    type Inner = u64;
}

impl fmt::Display for ExtraServiceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for ExtraServiceId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for ExtraServiceId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtraServiceEvent {
    Create {
        id: ExtraServiceId,
        name: String,
        description: String,
        price: Price,
    },
    NameChanged {
        id: ExtraServiceId,
        name: String,
    },
    DescriptionChanged {
        id: ExtraServiceId,
        description: String,
    },
    PriceChanged {
        id: ExtraServiceId,
        price: Price,
    },
}

impl Event for ExtraServiceEvent {
    type Id = ExtraServiceId;
}

#[derive(Debug, Default, Clone)]
pub struct ExtraService {
    id: ExtraServiceId,
    name: String,
    description: String,
    price: Price,

    events: EventQueue<ExtraServiceEvent>,
}

impl ExtraService {
    pub fn create(
        id: ExtraServiceId,
        name: String,
        description: String,
        price: Price,
    ) -> Result<Self, ExtraServiceError> {
        let mut entity = ExtraService::default();
        let event = ExtraServiceEvent::Create {
            id,
            name,
            description,
            price,
        };
        entity.validate(&event)?;
        entity.apply(event);
        Ok(entity)
    }

    pub fn change_name(&mut self, name: String) -> Result<(), ExtraServiceError> {
        let event = ExtraServiceEvent::NameChanged { id: self.id, name };
        self.validate(&event)?;
        self.apply(event);
        Ok(())
    }

    pub fn change_description(&mut self, description: String) {
        let event = ExtraServiceEvent::DescriptionChanged {
            id: self.id,
            description,
        };
        self.apply(event);
    }

    pub fn change_price(&mut self, price: Price) {
        let event = ExtraServiceEvent::PriceChanged { id: self.id, price };
        self.apply(event);
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn price(&self) -> &Price {
        &self.price
    }
}

impl Entity for ExtraService {
    type Id = ExtraServiceId;
    type Event = ExtraServiceEvent;
    type Error = ExtraServiceError;

    fn id(&self) ->  Self::Id {
        self.id
    }

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        let validate_name = |name: &str| {
            if name.trim().is_empty() {
                Err(ExtraServiceError::NameIsEmpty)
            } else {
                Ok(())
            }
        };
        match event {
            ExtraServiceEvent::Create { name, .. } => validate_name(&name),
            ExtraServiceEvent::NameChanged { name, .. } => validate_name(&name),
            ExtraServiceEvent::DescriptionChanged { .. } => Ok(()),
            ExtraServiceEvent::PriceChanged { .. } => Ok(()),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        if let Err(_) = self.validate(&event) {
            return;
        }
        match event.clone() {
            ExtraServiceEvent::Create {
                id,
                name,
                description,
                price,
            } => {
                if self.id == id {
                    return;
                }
                self.id = id;
                self.name = name;
                self.description = description;
                self.price = price;
            }
            ExtraServiceEvent::NameChanged { id, name, .. } => {
                if self.id != id {
                    return;
                }
                self.name = name;
            }
            ExtraServiceEvent::DescriptionChanged {
                id, description, ..
            } => {
                if self.id != id {
                    return;
                }
                self.description = description;
            }
            ExtraServiceEvent::PriceChanged { id, price, .. } => {
                if self.id != id {
                    return;
                }
                self.price = price;
            }
        }
        self.events.push(event);
    }

    fn entity_name() -> &'static str {
        "extra_service"
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl IntoIterator for ExtraService {
    type Item = ExtraServiceEvent;
    type IntoIter = EventQueueIntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
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

#[derive(Error, Debug)]
pub enum ExtraServiceError {
    #[error("Name cannot be empty")]
    NameIsEmpty,
}

impl From<ExtraServiceError> for DataAccessError {
    fn from(value: ExtraServiceError) -> Self {
        Self::ClientSideError(Box::new(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Price {
    amount: u64,
    currency: Currency,
}

impl Price {
    pub fn new(amount: u64, currency: Currency) -> Price {
        Price { amount, currency }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.currency {
            Currency::JPY => write!(f, "¥{}", self.amount.to_formatted_string(&Locale::ja)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Currency {
    #[default]
    JPY,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extra_service_create() {
        let service = ExtraService::create(
            ExtraServiceId(0),
            "AF".to_owned(),
            "アナルセックスを指します。".to_owned(),
            Price::new(10000, Currency::JPY),
        )
        .unwrap();
        assert_ne!(service.id(), ExtraServiceId(0));
        assert_eq!(service.name(), "AF");
        assert_eq!(service.description(), "アナルセックスを指します。");
        assert_eq!(service.price(), &Price::new(10000, Currency::JPY));
    }

    #[test]
    fn test_price_display() {
        let price = Price::new(1000000, Currency::JPY);
        assert_eq!(format!("{}", price), "¥1,000,000");
    }
}
