use std::{error::Error, ops::Range};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dely::{
    domain::{
        core::{
            CoreEvent, ExtraService, ExtraServiceEvent, Media, MediaEvent, Prostitute,
            ProstituteEvent, ProstituteId, Schedule, ScheduleEvent, ScheduleId, Shift, ShiftId,
            ShiftStatus,
        },
        Aggregation, Entity,
    },
    DelyConfig,
};
use eventstore::{ClientSettings, Position, StreamPosition, SubscribeToAllOptions};
use meilisearch_sdk::{task_info::TaskInfo, tasks::Task};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info, log::warn, Level};
use uuid::Uuid;

static VSERSION_UID: &str = "eventstore_version";

#[tokio::main]
async fn main() {
    match DelyConfig::load() {
        Ok(config) => {
            tracing_subscriber::fmt()
                .with_max_level(Level::from(&config.logger.level))
                .init();
            if let Err(error) = subscribe(&config).await {
                error!("アプリケーションエラー: {}", error);
            }
        }
        Err(error) => {
            tracing_subscriber::fmt::init();
            error!("アプリケーションエラー: {}", error)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct EventstoreVersion {
    id: u64,
    event_id: Uuid,
    position: Position,
}

async fn subscribe(config: &DelyConfig) -> Result<(), Box<dyn Error>> {
    let settings = config.eventstore.url.parse::<ClientSettings>()?;
    let mut client = Client {
        eventstore: eventstore::Client::new(settings)?,
        meilisearch: meilisearch_sdk::Client::new(
            &config.meilisearch.url,
            &config.meilisearch.api_key,
        ),
        task_info: None,
    };
    let version = client
        .meilisearch
        .index(VSERSION_UID)
        .get_document::<EventstoreVersion>("1")
        .await?;
    let mut sub = client
        .eventstore
        .subscribe_to_all(
            &SubscribeToAllOptions::default().position(StreamPosition::Position(version.position)),
        )
        .await;
    loop {
        match sub.next().await {
            Ok(resolved) => {
                if let Ok(core_event) = CoreEvent::try_from(&resolved) {
                    info!("ドメインイベントを受信: {:?}", core_event);
                    if let Err(e) = client.execute(core_event).await {
                        error!("イベント実行エラー: {}", e);
                        continue;
                    }
                } else {
                    info!("システムイベントを受信: {:?}", resolved);
                }
                let event = resolved.get_original_event();
                if let Err(e) = client
                    .meilisearch
                    .index(VSERSION_UID)
                    .add_documents(
                        &[EventstoreVersion {
                            id: 1,
                            event_id: event.id,
                            position: event.position,
                        }],
                        Some("id"),
                    )
                    .await
                {
                    error!("バージョン情報保存失敗: {}", e);
                    // TODO: バージョン情報をローカルに保存する等必要
                }
            }
            Err(e) => return Err(Box::new(e)),
        }
    }
}

#[async_trait]
pub trait Execute<E> {
    type Error: Error;
    async fn execute(&mut self, event: E) -> Result<(), Self::Error>;
}

struct Client {
    eventstore: eventstore::Client,
    meilisearch: meilisearch_sdk::Client,
    task_info: Option<TaskInfo>,
}

impl Client {
    async fn wait_for_completion(&self) -> Result<Option<Task>, meilisearch_sdk::errors::Error> {
        if let Some(task_info) = &self.task_info {
            loop {
                match self.meilisearch.wait_for_task(task_info, None, None).await {
                    Ok(task) => match task {
                        Task::Succeeded { .. } | Task::Failed { .. } => return Ok(Some(task)),
                        _ => continue,
                    },
                    Err(meilisearch_sdk::errors::Error::Timeout) => continue,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(None)
    }
}

#[async_trait]
impl Execute<CoreEvent> for Client {
    type Error = meilisearch_sdk::errors::Error;
    async fn execute(&mut self, event: CoreEvent) -> Result<(), Self::Error> {
        Ok(match event {
            CoreEvent::ExtraServiceEvent(event) => self.execute(event).await?,
            CoreEvent::MediaEvent(event) => self.execute(event).await?,
            CoreEvent::ProstituteEvent(event) => self.execute(event).await?,
            CoreEvent::ScheduleEvent(event) => self.execute(event).await?,
        })
    }
}

#[async_trait]
impl Execute<ExtraServiceEvent> for Client {
    type Error = meilisearch_sdk::errors::Error;
    async fn execute(&mut self, event: ExtraServiceEvent) -> Result<(), Self::Error> {
        let index = self.meilisearch.index(ExtraService::ENTITY_NAME);
        let task = match event {
            ExtraServiceEvent::Created {
                id,
                name,
                description,
                price,
            } => {
                if let Ok(entity) = ExtraService::create(id, name, description, price) {
                    index.add_documents(&[entity], Some("id")).await?
                } else {
                    warn!("不正なエンティティの登録をスキップしました");
                    return Ok(());
                }
            }
            ExtraServiceEvent::NameChanged { .. }
            | ExtraServiceEvent::DescriptionChanged { .. }
            | ExtraServiceEvent::PriceChanged { .. } => {
                index.add_or_update(&[event], Some("id")).await?
            }
            ExtraServiceEvent::Deleted { id } => index.delete_document(id).await?,
        };
        self.task_info = Some(task);
        Ok(())
    }
}

#[async_trait]
impl Execute<MediaEvent> for Client {
    type Error = meilisearch_sdk::errors::Error;
    async fn execute(&mut self, event: MediaEvent) -> Result<(), Self::Error> {
        let index = self.meilisearch.index(Media::ENTITY_NAME);
        let task = match event {
            MediaEvent::Created { id, mime, data } => {
                if let Ok(entity) = Media::create(id, mime, data) {
                    index.add_documents(&[entity], Some("id")).await?
                } else {
                    warn!("不正なエンティティの登録をスキップしました");
                    return Ok(());
                }
            }
            MediaEvent::Deleted { id } => index.delete_document(id).await?,
        };
        self.task_info = Some(task);
        Ok(())
    }
}

#[async_trait]
impl Execute<ProstituteEvent> for Client {
    type Error = meilisearch_sdk::errors::Error;
    async fn execute(&mut self, event: ProstituteEvent) -> Result<(), Self::Error> {
        let uid = Prostitute::ENTITY_NAME;
        let index = self.meilisearch.index(uid);
        let task = match event {
            ProstituteEvent::ProstituteJoined {
                id,
                name,
                catchphrase,
                profile,
                message,
                figure,
                blood,
                birthday,
                questions,
                images,
                video,
            } => {
                if let Ok(entity) = Prostitute::join(
                    id,
                    name,
                    catchphrase,
                    profile,
                    message,
                    figure,
                    blood,
                    birthday,
                    questions,
                    images,
                    video,
                ) {
                    index.add_documents(&[entity], Some("id")).await?
                } else {
                    warn!("不正なエンティティの登録をスキップしました");
                    return Ok(());
                }
            }
            ProstituteEvent::ProstituteRejoined { id } => {
                index
                    .add_or_update(&[json!({"id": id, "leaved": false})], Some("id"))
                    .await?
            }
            ProstituteEvent::ProstituteLeaved { id } => {
                index
                    .add_or_update(&[json!({"id": id, "leaved": true})], Some("id"))
                    .await?
            }
            ProstituteEvent::NameChanged { .. }
            | ProstituteEvent::CatchphraseChanged { .. }
            | ProstituteEvent::ProfileChanged { .. }
            | ProstituteEvent::MessageChanged { .. }
            | ProstituteEvent::FigureChanged { .. }
            | ProstituteEvent::BloodTypeChanged { .. }
            | ProstituteEvent::BirthdayChanged { .. }
            | ProstituteEvent::QuestionsChanged { .. }
            | ProstituteEvent::ImagesChanged { .. }
            | ProstituteEvent::VideoChanged { .. } => {
                index.add_or_update(&[event], Some("id")).await?
            }
            ProstituteEvent::QuestionAdded { id, .. }
            | ProstituteEvent::QuestionDeleted { id, .. }
            | ProstituteEvent::QuestionSwapped { id, .. }
            | ProstituteEvent::ImageAdded { id, .. }
            | ProstituteEvent::ImageDeleted { id, .. }
            | ProstituteEvent::ImageSwapped { id, .. } => {
                self.wait_for_completion().await?;
                let mut entity = index.get_document::<Prostitute>(&id.to_string()).await?;
                entity.apply(event);
                index.add_or_update(&[entity], Some("id")).await?
            }
            ProstituteEvent::ProstituteDeleted { id } => index.delete_document(id).await?,
        };
        self.task_info = Some(task);
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct MeiliSchedule {
    id: ScheduleId,
    prostitute_id: ProstituteId,
}

#[derive(Default, Serialize, Deserialize)]
pub struct MeiliShift {
    id: ShiftId,
    schedule_id: Option<ScheduleId>,
    time: Option<Range<DateTime<Utc>>>,
    status: Option<ShiftStatus>,
}

#[async_trait]
impl Execute<ScheduleEvent> for Client {
    type Error = meilisearch_sdk::errors::Error;
    async fn execute(&mut self, event: ScheduleEvent) -> Result<(), Self::Error> {
        let index_schedule = self.meilisearch.index(Schedule::ENTITY_NAME);
        let index_shift = self.meilisearch.index(Shift::ENTITY_NAME);
        let task = match event {
            ScheduleEvent::ScheduleCreated { id, prostitute_id } => {
                index_schedule
                    .add_documents(&[MeiliSchedule { id, prostitute_id }], Some("id"))
                    .await?
            }
            ScheduleEvent::ScheduleDeleted { id } => index_schedule.delete_document(id).await?,
            ScheduleEvent::ShiftAdded { id, shift } => {
                index_shift
                    .add_documents(
                        &[MeiliShift {
                            id: shift.id(),
                            schedule_id: Some(id),
                            time: Some(shift.time()),
                            status: Some(shift.status()),
                        }],
                        Some("id"),
                    )
                    .await?
            }
            ScheduleEvent::ShiftTimeChanged { shift_id, time } => {
                index_shift
                    .add_or_update(
                        &[MeiliShift {
                            id: shift_id,
                            time: Some(time),
                            ..Default::default()
                        }],
                        Some("id"),
                    )
                    .await?
            }
            ScheduleEvent::ShiftStatusChanged { shift_id, status } => {
                index_shift
                    .add_or_update(
                        &[MeiliShift {
                            id: shift_id,
                            status: Some(status),
                            ..Default::default()
                        }],
                        Some("id"),
                    )
                    .await?
            }
            ScheduleEvent::ShiftsDeleted { shift_ids } => {
                index_shift.delete_documents(&shift_ids).await?
            }
        };
        self.task_info = Some(task);
        Ok(())
    }
}
