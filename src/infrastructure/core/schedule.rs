use async_trait::async_trait;
use eventstore::{AppendToStreamOptions, Client, EventData, ExpectedRevision, ResolvedEvent};

use crate::domain::core::{
    Schedule, ScheduleEvent, ScheduleId, ScheduleRepository,
};
use crate::domain::{DataAccessError, Aggregation, Entity};
use crate::infrastructure::{EventConvertError, stream_name};
use crate::infrastructure::{from_event, try_from_resolved_event};

#[derive(Clone)]
pub struct EventStoreScheduleRepository {
    client: Client,
}

impl EventStoreScheduleRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ScheduleRepository for EventStoreScheduleRepository {
    async fn find_by_id(
        &self,
        id: ScheduleId,
    ) -> Result<Option<Schedule>, DataAccessError> {
        match self
            .client
            .read_stream(stream_name::<Schedule>(id), &Default::default())
            .await
        {
            Ok(mut stream) => {
                let mut entity = Schedule::default();
                loop {
                    match stream.next().await {
                        Ok(Some(e)) => entity.apply(TryFrom::try_from(&e)?),
                        Ok(_) => break,
                        Err(eventstore::Error::ResourceDeleted) => return Ok(None),
                        Err(eventstore::Error::ResourceNotFound) => return Ok(None),
                        Err(e) => return Err(e.into()),
                    }
                }
                if let None = entity.peek() {
                    Ok(None)
                } else {
                    entity.clear();
                    Ok(Some(entity))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn save(&mut self, entity: &mut Schedule) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<Schedule>(entity.id());
        let rev = match entity.peek() {
            Some(ScheduleEvent::ScheduleCreated { .. }) => ExpectedRevision::NoStream,
            Some(_) => ExpectedRevision::StreamExists,
            None => return Ok(false),
        };
        self.client
            .append_to_stream(
                &stream_name,
                &AppendToStreamOptions::default().expected_revision(rev),
                entity
                    .pop_all()
                    .into_iter()
                    .map(EventData::from)
                    .collect::<Vec<_>>(),
            )
            .await?;
        Ok(true)
    }

    async fn delete(&mut self, entity: &mut Schedule) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<Schedule>(entity.id());
        self.client.append_to_stream(
            &stream_name,
            &AppendToStreamOptions::default().expected_revision(ExpectedRevision::StreamExists),
            EventData::from(ScheduleEvent::ScheduleDeleted { id: entity.id() }),
        ).await?;
        self.client
            .delete_stream(&stream_name, &Default::default())
            .await?;
        Ok(true)
    }
}

impl From<ScheduleEvent> for EventData {
    fn from(value: ScheduleEvent) -> Self {
        from_event(value)
    }
}

impl TryFrom<&ResolvedEvent> for ScheduleEvent {
    type Error = EventConvertError;

    fn try_from(value: &ResolvedEvent) -> Result<Self, Self::Error> {
        try_from_resolved_event(value)
    }
}
