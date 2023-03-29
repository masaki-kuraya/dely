mod extra_service;
mod media;
mod prostitute;
mod schedule;

use eventstore::ResolvedEvent;

use crate::domain::{
    core::{CoreEvent, ExtraService, Media, Prostitute, Schedule},
    Entity,
};

pub use self::extra_service::*;
pub use self::media::*;
pub use self::prostitute::*;
pub use self::schedule::*;

use super::EventConvertError;

impl TryFrom<&ResolvedEvent> for CoreEvent {
    type Error = EventConvertError;

    fn try_from(value: &ResolvedEvent) -> Result<Self, Self::Error> {
        let x = value
            .get_original_stream_id()
            .split("-")
            .next()
            .ok_or(EventConvertError)?;
        match x {
            ExtraService::ENTITY_NAME => {
                Ok(CoreEvent::ExtraServiceEvent(TryFrom::try_from(value)?))
            }
            Media::ENTITY_NAME => Ok(CoreEvent::MediaEvent(TryFrom::try_from(value)?)),
            Prostitute::ENTITY_NAME => Ok(CoreEvent::ProstituteEvent(TryFrom::try_from(value)?)),
            Schedule::ENTITY_NAME => Ok(CoreEvent::ScheduleEvent(TryFrom::try_from(value)?)),
            _ => Err(EventConvertError),
        }
    }
}
