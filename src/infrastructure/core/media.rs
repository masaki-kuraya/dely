use async_trait::async_trait;
use bytes::Bytes;
use eventstore::{
    AppendToStreamOptions, Client, DeleteStreamOptions, EventData, ExpectedRevision, ResolvedEvent,
};
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;

use crate::domain::core::{Media, MediaEvent, MediaId, MediaRepository};
use crate::domain::{DataAccessError, Entity};
use crate::infrastructure::{entity_id, stream_name};
use crate::infrastructure::EventConvertError;

#[derive(Clone)]
pub struct EventStoreMediaRepository {
    client: Client,
}

impl EventStoreMediaRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MediaRepository for EventStoreMediaRepository {
    async fn find_by_id(&self, id: MediaId) -> Result<Option<Media>, DataAccessError> {
        match self
            .client
            .read_stream(stream_name::<Media>(id), &Default::default())
            .await
        {
            Ok(mut stream) => {
                let mut entity = Media::default();
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

    async fn save(&mut self, entity: &mut Media) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<Media>(entity.id());
        let rev = match entity.peek() {
            Some(MediaEvent::Create { .. }) => ExpectedRevision::NoStream,
            None => return Ok(false),
        };
        let mut events = Vec::new();
        while let Some(e) = entity.pop() {
            events.push(EventData::from(e))
        }
        self.client
            .append_to_stream(
                &stream_name,
                &AppendToStreamOptions::default().expected_revision(rev),
                events,
            )
            .await?;
        Ok(true)
    }

    async fn delete(&mut self, entity: &mut Media) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<Media>(entity.id());
        self.client
            .delete_stream(stream_name, &DeleteStreamOptions::default())
            .await?;
        Ok(true)
    }
}

impl From<MediaEvent> for EventData {
    fn from(value: MediaEvent) -> Self {
        match value {
            MediaEvent::Create { mime, data, .. } => {
                let mut meta = HashMap::new();
                meta.insert("contentType".to_owned(), mime.to_string());
                EventData::binary("Create", Bytes::from(data))
                    .metadata_as_json(meta)
                    .unwrap()
            }
        }
    }
}

impl TryFrom<ResolvedEvent> for MediaEvent {
    type Error = EventConvertError;

    fn try_from(value: ResolvedEvent) -> Result<Self, Self::Error> {
        let event = value.get_original_event();
        match event.event_type.borrow() {
            "Create" => Ok(MediaEvent::Create {
                id: entity_id(&event.stream_id).ok_or(EventConvertError)?,
                mime: serde_json::from_slice::<Value>(&event.custom_metadata)?
                    .as_object()
                    .into_iter()
                    .filter_map(|v| v.get("contentType"))
                    .filter_map(Value::as_str)
                    .find_map(|s| s.parse().ok())
                    .ok_or(EventConvertError)?,
                data: event.data.to_vec(),
            }),
            _ => Err(EventConvertError),
        }
    }
}

#[cfg(test)]
mod tests {
    use eventstore::Client;

    use crate::domain::{
        core::{Media, MediaRepository},
        ID_GENERATOR,
    };

    use super::EventStoreMediaRepository;

    #[tokio::test]
    async fn test_repository() {
        // リポジトリ作成
        let settings = "esdb://localhost:2113?tls=false".parse().unwrap();
        let client = Client::new(settings).unwrap();
        let mut repo = EventStoreMediaRepository::new(client.clone());

        // エンティティ生成
        let id = ID_GENERATOR.generate().await;
        let mut entity = Media::create(
            id,
            "image/gif".parse().unwrap(),
            b"\x47\x49\x46\x38\x39\x61\x01\x00\x01\x00\xF0\x00\x00\xFF\xFF\xFF\x00\x00\x00\x2C\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x44\x01\x00\x3B".to_vec(),
        )
        .unwrap();

        // エンティティ登録確認
        assert_eq!(repo.save(&mut entity).await.unwrap(), true);
        assert_eq!(
            repo.find_by_id(id).await.unwrap(),
            Media::create(
                id,
                "image/gif".parse().unwrap(),
                b"\x47\x49\x46\x38\x39\x61\x01\x00\x01\x00\xF0\x00\x00\xFF\xFF\xFF\x00\x00\x00\x2C\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x44\x01\x00\x3B".to_vec()
            )
            .ok()
        );
        // エンティティ削除確認
        assert_eq!(repo.delete(&mut entity).await.unwrap(), true);
        assert_eq!(repo.find_by_id(id).await.unwrap(), None);
    }
}
