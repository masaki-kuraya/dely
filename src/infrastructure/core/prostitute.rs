use async_trait::async_trait;
use eventstore::{AppendToStreamOptions, Client, EventData, ExpectedRevision, ResolvedEvent};

use crate::domain::core::{
    Prostitute, ProstituteEvent, ProstituteId, ProstituteRepository,
};
use crate::domain::{DataAccessError, Entity};
use crate::infrastructure::{EventConvertError, stream_name};
use crate::infrastructure::{from_event, try_from_resolved_event};

#[derive(Clone)]
pub struct EventStoreProstituteRepository {
    client: Client,
}

impl EventStoreProstituteRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ProstituteRepository for EventStoreProstituteRepository {
    async fn find_by_id(
        &self,
        id: ProstituteId,
    ) -> Result<Option<Prostitute>, DataAccessError> {
        match self
            .client
            .read_stream(stream_name::<Prostitute>(id), &Default::default())
            .await
        {
            Ok(mut stream) => {
                let mut entity = Prostitute::default();
                loop {
                    match stream.next().await {
                        Ok(Some(e)) => entity.apply(e.try_into()?),
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

    async fn save(&mut self, entity: &mut Prostitute) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<Prostitute>(entity.id());
        let rev = match entity.peek() {
            Some(ProstituteEvent::Joined { .. }) => ExpectedRevision::NoStream,
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

    async fn delete(&mut self, entity: &mut Prostitute) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<Prostitute>(entity.id());
        self.client
            .delete_stream(&stream_name, &Default::default())
            .await?;
        Ok(true)
    }
}

impl From<ProstituteEvent> for EventData {
    fn from(value: ProstituteEvent) -> Self {
        from_event(value)
    }
}

impl TryFrom<ResolvedEvent> for ProstituteEvent {
    type Error = EventConvertError;

    fn try_from(value: ResolvedEvent) -> Result<Self, Self::Error> {
        try_from_resolved_event(value)
    }
}