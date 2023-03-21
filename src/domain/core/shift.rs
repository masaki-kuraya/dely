use std::{convert::Infallible, fmt, ops::Deref};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::{Entity, Event, EventQueue, EventQueueIntoIter, Id};

use super::ProstituteId;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub struct ShiftId(u64);

impl Id for ShiftId {
    type Inner = u64;
}

impl fmt::Display for ShiftId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for ShiftId {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for ShiftId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShiftEvent {
    Created {
        id: ShiftId,
        prostitute_id: ProstituteId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    },
    Updated {
        id: ShiftId,
        prostitute_id: Option<ProstituteId>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    },
}

impl Event for ShiftEvent {
    type Id = ShiftId;
}

#[derive(Debug, Clone, Default)]
pub struct Shift {
    id: ShiftId,
    prostitute_id: ProstituteId,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,

    events: EventQueue<ShiftEvent>,
}

impl Shift {
    pub fn create(
        id: ShiftId,
        prostitute_id: ProstituteId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Self, Infallible> {
        let mut entity = Shift::default();
        let event = ShiftEvent::Created {
            id,
            prostitute_id,
            start_time,
            end_time,
        };
        entity.validate(&event)?;
        entity.apply(event);
        Ok(entity)
    }
}

impl Entity for Shift {
    type Id = ShiftId;
    type Event = ShiftEvent;
    type Error = Infallible;

    fn id(&self) -> ShiftId {
        self.id
    }

    fn validate(&self, _event: &Self::Event) -> Result<(), Self::Error> {
        Ok(())
    }

    fn apply(&mut self, event: Self::Event) {
        if let Err(_) = self.validate(&event) {
            return;
        }
        match event {
            ShiftEvent::Created {
                id,
                prostitute_id,
                start_time,
                end_time,
            } => {
                if self.id == id {
                    return;
                }
                self.id = id;
                self.prostitute_id = prostitute_id;
                self.start_time = start_time;
                self.end_time = end_time;
            }
            ShiftEvent::Updated {
                id,
                prostitute_id,
                start_time,
                end_time,
            } => {
                if self.id != id {
                    return;
                }
                if let Some(x) = prostitute_id {
                    self.prostitute_id = x;
                }
                if let Some(x) = start_time {
                    self.start_time = x;
                }
                if let Some(x) = end_time {
                    self.end_time = x;
                }
            }
        }
        self.events.push(event);
    }

    fn entity_name() -> &'static str {
        "shift"
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl IntoIterator for Shift {
    type Item = ShiftEvent;
    type IntoIter = EventQueueIntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

impl PartialEq for Shift {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.prostitute_id == other.prostitute_id
            && self.start_time == other.start_time
            && self.end_time == other.end_time
    }
}
