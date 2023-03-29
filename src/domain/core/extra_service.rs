use std::fmt;

use async_trait::async_trait;
use derive_more::{Deref, Display, Error, From, IntoIterator};
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

#[async_trait]
pub trait ExtraServiceRepository {
    async fn find_by_id(&self, id: ExtraServiceId)
        -> Result<Option<ExtraService>, DataAccessError>;
    async fn save(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError>;
    async fn delete(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError>;
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ExtraServiceId(u64);

impl Id for ExtraServiceId {
    type Inner = u64;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtraServiceEvent {
    Created {
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
    Deleted {
        id: ExtraServiceId,
    },
}

impl Event for ExtraServiceEvent {
    type Id = ExtraServiceId;
}

#[derive(Debug, Default, Clone, IntoIterator, Serialize, Deserialize)]
pub struct ExtraService {
    id: ExtraServiceId,
    name: String,
    description: String,
    price: Price,
    #[serde(skip)]
    #[into_iterator]
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
        let event = ExtraServiceEvent::Created {
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

    const ENTITY_NAME: &'static str = "extra_service";

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Aggregation for ExtraService {
    type Event = ExtraServiceEvent;
    type Error = ExtraServiceError;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        let validate_name = |name: &str| {
            if name.trim().is_empty() {
                Err(ExtraServiceError::NameIsEmpty)
            } else {
                Ok(())
            }
        };
        match event {
            ExtraServiceEvent::Created { name, .. } => validate_name(&name),
            ExtraServiceEvent::NameChanged { name, .. } => validate_name(&name),
            ExtraServiceEvent::DescriptionChanged { .. } => Ok(()),
            ExtraServiceEvent::PriceChanged { .. } => Ok(()),
            ExtraServiceEvent::Deleted { .. } => Ok(()),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        if let Err(_) = self.validate(&event) {
            return;
        }
        match event.clone() {
            ExtraServiceEvent::Created {
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
            ExtraServiceEvent::Deleted { .. } => {}
        }
        self.events.push(event);
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

#[derive(Error, Display, Debug)]
pub enum ExtraServiceError {
    #[display(fmt = "Name cannot be empty")]
    NameIsEmpty,
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
            ExtraServiceId(30),
            "AF".to_owned(),
            "アナルセックスを指します。".to_owned(),
            Price::new(10000, Currency::JPY),
        )
        .unwrap();
        assert_eq!(service.id(), ExtraServiceId(30));
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
