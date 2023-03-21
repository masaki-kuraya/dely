pub mod core;

use eventstore::{EventData, ResolvedEvent};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::domain::{DataAccessError, Event, Id, Entity};

use std::{fmt::Display, str::FromStr};

impl From<eventstore::Error> for DataAccessError {
    fn from(value: eventstore::Error) -> Self {
        match value {
            eventstore::Error::ConnectionClosed
            | eventstore::Error::Grpc { .. }
            | eventstore::Error::GrpcConnectionError(_)
            | eventstore::Error::DeadlineExceeded
            | eventstore::Error::InitializationError(_) => Self::ConnectionError(Box::new(value)),
            eventstore::Error::ServerError(_)
            | eventstore::Error::NotLeaderException(_)
            | eventstore::Error::AccessDenied
            | eventstore::Error::UnsupportedFeature
            | eventstore::Error::InternalParsingError(_)
            | eventstore::Error::InternalClientError => Self::QueryError(Box::new(value)),
            eventstore::Error::ResourceNotFound | eventstore::Error::ResourceDeleted => {
                Self::ReadError(Box::new(value))
            }
            eventstore::Error::ResourceAlreadyExists
            | eventstore::Error::WrongExpectedVersion { .. } => Self::WriteError(Box::new(value)),
            eventstore::Error::IllegalStateError(_) => Self::ClientSideError(Box::new(value)),
        }
    }
}

impl From<EventConvertError> for DataAccessError {
    fn from(value: EventConvertError) -> Self {
        DataAccessError::ClientSideError(Box::new(value))
    }
}

#[derive(Debug)]
pub struct EventConvertError;

impl std::error::Error for EventConvertError {}

impl Display for EventConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to convert event")
    }
}

impl From<serde_json::Error> for EventConvertError {
    fn from(_value: serde_json::Error) -> Self {
        EventConvertError
    }
}

fn entity_id<I, T>(stream_id: &str) -> Option<I>
where
    I: Id<Inner = T>,
    T: FromStr,
{
    stream_id
        .split('-')
        .filter_map(|s| s.parse::<T>().ok())
        .map(I::from)
        .last()
}

fn stream_name<E: Entity>(id: E::Id) -> String {
    E::entity_name().to_owned() + "-" + &id.to_string()
}

fn from_event<E: Event>(event: E) -> EventData {
    let root = serde_json::to_value(event).unwrap();
    let event_type = root.as_object().unwrap().keys().next().unwrap();
    let mut data = root[event_type].clone();
    data.as_object_mut().unwrap().remove("id");
    EventData::json(event_type, data).unwrap()
}

fn try_from_resolved_event<E, I>(value: ResolvedEvent) -> Result<E, EventConvertError>
where
    E: DeserializeOwned + Event<Id = I>,
    I: Id,
{
    let event = value.get_original_event();
    let id = entity_id::<I, I::Inner>(&event.stream_id).ok_or(EventConvertError)?;
    let mut data: Value = serde_json::from_slice(event.data.as_ref())?;
    data.as_object_mut()
        .unwrap()
        .insert("id".to_owned(), json!(id));
    let json = json!({ &event.event_type: data });
    Ok(serde_json::from_value(json)?)
}
