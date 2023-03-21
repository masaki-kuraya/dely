use core::fmt;
use std::ops::Deref;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{DataAccessError, Entity, EventQueue, EventQueueIntoIter, Id, Event};

use super::Mime;

#[async_trait]
pub trait MediaRepository {
    async fn find_by_id(&self, id: MediaId) -> Result<Option<Media>, DataAccessError>;
    async fn save(&mut self, entity: &mut Media) -> Result<bool, DataAccessError>;
    async fn delete(&mut self, entity: &mut Media) -> Result<bool, DataAccessError>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MediaId(pub u64);

impl Id for MediaId {
    type Inner = u64;
}

impl fmt::Display for MediaId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for MediaId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for MediaId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaEvent {
    Create {
        id: MediaId,
        mime: Mime,
        data: Vec<u8>,
    },
}

impl Event for MediaEvent {
    type Id = MediaId;
}

#[derive(Debug, Default, Clone)]
pub struct Media {
    id: MediaId,
    mime: Mime,
    data: Vec<u8>,

    events: EventQueue<MediaEvent>,
}

impl Media {
    pub fn create(id: MediaId, mime: Mime, data: Vec<u8>) -> Result<Self, MediaError> {
        let mut entity = Media::default();
        let event = MediaEvent::Create { id, mime, data };
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
    type Event = MediaEvent;
    type Error = MediaError;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            MediaEvent::Create { data, .. } => {
                if data.is_empty() {
                    Err(MediaError::DataIsEmpty)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn apply(&mut self, event: Self::Event) {
        if let Err(_) = self.validate(&event) {
            return;
        }
        match event.clone() {
            MediaEvent::Create { id, mime, data } => {
                if self.id == id {
                    return;
                }
                self.id = id;
                self.mime = mime;
                self.data = data;
            }
        }
        self.events.push(event);
    }

    fn entity_name() -> &'static str {
        "media"
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }

}

impl IntoIterator for Media {
    type Item = MediaEvent;
    type IntoIter = EventQueueIntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl PartialEq for Media {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.mime == other.mime && self.data == other.data
    }
}

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("Data cannot be empty")]
    DataIsEmpty,
}

impl From<MediaError> for DataAccessError {
    fn from(value: MediaError) -> Self {
        Self::ClientSideError(Box::new(value))
    }
}
