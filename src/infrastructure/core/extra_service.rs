use async_trait::async_trait;
use eventstore::{AppendToStreamOptions, Client, EventData, ExpectedRevision, ResolvedEvent};

use crate::domain::core::{
    ExtraService, ExtraServiceEvent, ExtraServiceId, ExtraServiceRepository,
};
use crate::domain::{DataAccessError, Entity};
use crate::infrastructure::{EventConvertError, stream_name};
use crate::infrastructure::{from_event, try_from_resolved_event};

#[derive(Clone)]
pub struct EventStoreExtraServiceRepository {
    client: Client,
}

impl EventStoreExtraServiceRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ExtraServiceRepository for EventStoreExtraServiceRepository {
    async fn find_by_id(
        &self,
        id: ExtraServiceId,
    ) -> Result<Option<ExtraService>, DataAccessError> {
        match self
            .client
            .read_stream(stream_name::<ExtraService>(id), &Default::default())
            .await
        {
            Ok(mut stream) => {
                let mut entity = ExtraService::default();
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

    async fn save(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<ExtraService>(entity.id());
        let rev = match entity.peek() {
            Some(ExtraServiceEvent::Create { .. }) => ExpectedRevision::NoStream,
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

    async fn delete(&mut self, entity: &mut ExtraService) -> Result<bool, DataAccessError> {
        let stream_name = stream_name::<ExtraService>(entity.id());
        self.client
            .delete_stream(&stream_name, &Default::default())
            .await?;
        Ok(true)
    }
}

impl From<ExtraServiceEvent> for EventData {
    fn from(value: ExtraServiceEvent) -> Self {
        from_event(value)
    }
}

impl TryFrom<ResolvedEvent> for ExtraServiceEvent {
    type Error = EventConvertError;

    fn try_from(value: ResolvedEvent) -> Result<Self, Self::Error> {
        try_from_resolved_event(value)
    }
}

#[cfg(test)]
mod tests {
    use eventstore::{Client, EventData, Position, RecordedEvent, ResolvedEvent};
    use serde_json::json;

    use crate::domain::{
        core::{Currency, ExtraService, ExtraServiceEvent, ExtraServiceRepository, Price},
        ID_GENERATOR,
    };

    use super::EventStoreExtraServiceRepository;

    #[tokio::test]
    async fn test_repository() {
        // リポジトリ作成
        let settings = "esdb://localhost:2113?tls=false".parse().unwrap();
        let client = Client::new(settings).unwrap();
        let mut repo = EventStoreExtraServiceRepository::new(client.clone());

        let id = ID_GENERATOR.generate().await;

        // エンティティ生成
        let mut entity = ExtraService::create(
            id,
            "名前".to_owned(),
            "説明".to_owned(),
            Price::new(1500, Currency::JPY),
        )
        .unwrap();
        entity.change_name("サービス名改".to_owned()).unwrap();

        // エンティティ登録確認
        assert_eq!(repo.save(&mut entity).await.unwrap(), true);
        assert_eq!(
            repo.find_by_id(id).await.unwrap(),
            ExtraService::create(
                id,
                "サービス名改".to_owned(),
                "説明".to_owned(),
                Price::new(1500, Currency::JPY),
            )
            .ok()
        );

        // エンティティ削除確認
        assert_eq!(repo.delete(&mut entity).await.unwrap(), true);
        assert_eq!(repo.find_by_id(id).await.unwrap(), None);
    }

    #[test]
    fn test_event_data_from() {
        let event = ExtraServiceEvent::Create {
            id: 999.into(),
            name: "サービス名".to_owned(),
            description: "説明".to_owned(),
            price: Price::new(1500, Currency::JPY),
        };
        let expected = EventData::json(
            "Create",
            json!({
                "name": "サービス名",
                "description": "説明",
                "price": {
                    "amount": 1500,
                    "currency": "JPY",
                }
            }),
        )
        .unwrap();
        assert_eq!(
            format!("{:?}", EventData::from(event)),
            format!("{:?}", expected),
        );
    }

    #[test]
    fn test_event_try_from() {
        let data = json!({
            "name": "テストサービス",
            "description": "テスト説明です",
            "price": {
                "amount": 5000,
                "currency": "JPY",
            }
        });
        let event = ResolvedEvent {
            event: Some(RecordedEvent {
                stream_id: "extra_service-100".to_owned(),
                id: Default::default(),
                revision: Default::default(),
                event_type: "Create".to_owned(),
                data: serde_json::to_vec(&data).unwrap().into(),
                metadata: Default::default(),
                custom_metadata: Default::default(),
                is_json: Default::default(),
                position: Position {
                    commit: Default::default(),
                    prepare: Default::default(),
                },
                created: Default::default(),
            }),
            link: None,
            commit_position: None,
        };
        let expected = ExtraServiceEvent::Create {
            id: 100.into(),
            name: "テストサービス".to_owned(),
            description: "テスト説明です".to_owned(),
            price: Price::new(5000, Currency::JPY),
        };
        assert_eq!(ExtraServiceEvent::try_from(event).ok(), Some(expected));
    }
}
