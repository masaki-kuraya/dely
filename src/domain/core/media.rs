use async_trait::async_trait;
use derive_more::{Deref, Display, Error, From, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

use super::Mime;

#[async_trait]
pub trait MediaRepository {
    async fn find_by_id(&self, id: MediaId) -> Result<Option<Media>, DataAccessError>;
    async fn save(&mut self, entity: &mut Media) -> Result<bool, DataAccessError>;
    async fn delete(&mut self, entity: &mut Media) -> Result<bool, DataAccessError>;
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default, Hash
)]
pub struct MediaId(pub u64);

impl Id for MediaId {
    type Inner = u64;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaEvent {
    Created {
        id: MediaId,
        mime: Mime,
        data: Vec<u8>,
    },
    Deleted {
        id: MediaId,
    },
}

impl Event for MediaEvent {
    type Id = MediaId;
}

#[derive(Debug, Default, Clone, IntoIterator, Serialize, Deserialize)]
pub struct Media {
    id: MediaId,
    mime: Mime,
    #[serde(with = "base64")]
    data: Vec<u8>,
    #[serde(skip)]
    #[into_iterator]
    events: EventQueue<MediaEvent>,
}

impl Media {
    pub fn create(id: MediaId, mime: Mime, data: Vec<u8>) -> Result<Self, MediaError> {
        let mut entity = Media::default();
        let event = MediaEvent::Created { id, mime, data };
        entity.validate(&event)?;
        entity.apply(event);
        Ok(entity)
    }

    pub fn mime(&self) -> &Mime {
        &self.mime
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Entity for Media {
    type Id = MediaId;

    const ENTITY_NAME: &'static str = "media";

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Aggregation for Media {
    type Event = MediaEvent;
    type Error = MediaError;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            MediaEvent::Created { data, .. } => {
                if data.is_empty() {
                    Err(MediaError::DataIsEmpty)
                } else {
                    Ok(())
                }
            }
            MediaEvent::Deleted { .. } => Ok(()),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        if let Err(_) = self.validate(&event) {
            return;
        }
        match event.clone() {
            MediaEvent::Created { id, mime, data } => {
                if self.id == id {
                    return;
                }
                self.id = id;
                self.mime = mime;
                self.data = data;
            }
            MediaEvent::Deleted { .. } => {}
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

impl PartialEq for Media {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.mime == other.mime && self.data == other.data
    }
}

impl Eq for Media {}

#[derive(Error, Display, Debug)]
pub enum MediaError {
    #[display(fmt = "Data cannot be empty")]
    DataIsEmpty,
}

mod base64 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, v);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            base64.as_bytes(),
        )
        .map_err(|e| serde::de::Error::custom(e))
    }
}
