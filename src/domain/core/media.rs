use async_trait::async_trait;
use bytes::{Buf, Bytes};
use derive_more::{Deref, Display, Error, From, IntoIterator};
use image::{
    codecs::{gif::GifDecoder, jpeg::JpegDecoder, png::PngDecoder, webp::WebPDecoder},
    ImageDecoder, ImageFormat,
};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

use super::Mime;

/// メディアリポジトリ
#[async_trait]
pub trait MediaRepository {
    /// メディアをIDで検索する
    async fn find_by_id(&self, id: MediaId) -> Result<Option<Media>, DataAccessError>;
    /// メディアを保存する
    async fn save(&mut self, entity: &mut Media) -> Result<bool, DataAccessError>;
    /// メディアを削除する
    async fn delete(&mut self, entity: &mut Media) -> Result<bool, DataAccessError>;
}

/// メディアID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default, Hash,
)]
pub struct MediaId(pub u64);

impl Id for MediaId {
    type Inner = u64;
}

/// メディアイベント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaEvent {
    MediaCreated {
        id: MediaId,
        mime: Mime,
        data: Bytes,
    },
    MediaDeleted {
        id: MediaId,
    },
}

impl Event for MediaEvent {
    type Id = MediaId;
}

/// メディア
#[derive(Debug, Default, Clone, IntoIterator, Serialize, Deserialize)]
pub struct Media {
    id: MediaId,
    mime: Mime,
    data: Bytes,
    #[serde(skip)]
    #[into_iterator]
    events: EventQueue<MediaEvent>,
}

impl Media {
    pub fn create<B: Into<Bytes>>(id: MediaId, data: B) -> Result<Self, MediaError> {
        let data = data.into();
        let mime = Self::validate_created(&data)?;
        let mut entity = Media {
            id,
            mime: mime.clone(),
            data: data.clone(),
            ..Media::default()
        };
        entity
            .events
            .push(MediaEvent::MediaCreated { id, mime, data });
        Ok(entity)
    }

    pub fn mime(&self) -> &Mime {
        &self.mime
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn validate_id(&self, id: &MediaId) -> Result<(), MediaError> {
        match self.id == *id {
            true => Ok(()),
            false => Err(MediaError::MismatchedId),
        }
    }

    fn validate_created(data: &Bytes) -> Result<Mime, MediaError> {
        match image::guess_format(data) {
            Ok(format) => match format {
                ImageFormat::Jpeg => {
                    if let Ok(decoder) = JpegDecoder::new(data.clone().reader()) {
                        if Self::is_image(&decoder) {
                            return Ok(Mime::IMAGE_JPEG);
                        }
                    }
                }
                ImageFormat::Png => {
                    if let Ok(decoder) = PngDecoder::new(data.clone().reader()) {
                        if Self::is_image(&decoder) {
                            return Ok(Mime::IMAGE_PNG);
                        }
                    }
                }
                ImageFormat::Gif => {
                    if let Ok(decoder) = GifDecoder::new(data.clone().reader()) {
                        if Self::is_image(&decoder) {
                            return Ok(Mime::IMAGE_GIF);
                        }
                    }
                }
                ImageFormat::WebP => {
                    if let Ok(decoder) = WebPDecoder::new(data.clone().reader()) {
                        if Self::is_image(&decoder) {
                            return Ok("image/webp".parse().unwrap());
                        }
                    }
                }
                _ => return Err(MediaError::UnsupportedFormat),
            },
            Err(_) => {
                if let Ok(ctx) = mp4parse::read_mp4(&mut data.clone().reader()) {
                    for t in ctx.tracks {
                        match t.track_type {
                            mp4parse::TrackType::Video => return Ok("video/mp4".parse().unwrap()),
                            _ => (),
                        }
                    }
                }
                return Err(MediaError::UnsupportedFormat);
            }
        }
        Err(MediaError::UnsupportedFormat)
    }

    fn is_image<D: for<'a> ImageDecoder<'a>>(decoder: &D) -> bool {
        let dimensions = decoder.dimensions();
        dimensions.0 > 0 && dimensions.1 > 0
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
            MediaEvent::MediaCreated { data, .. } => {
                Self::validate_created(data)?;
                Ok(())
            }
            MediaEvent::MediaDeleted { id } => self.validate_id(id),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            MediaEvent::MediaCreated { id, data, .. } => {
                if self.id != id {
                    if let Ok(entity) = Self::create(id, data) {
                        *self = entity;
                    }
                }
            }
            MediaEvent::MediaDeleted { .. } => {}
        }
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

/// メディアエラー
#[derive(Error, Display, Debug)]
pub enum MediaError {
    /// IDが一致しません
    #[display(fmt = "ID does not match")]
    MismatchedId,
    /// データが空値です
    #[display(fmt = "Data cannot be empty")]
    DataIsEmpty,
    /// サポートされていないメディア形式です
    #[display(fmt = "Unsupported media format")]
    UnsupportedFormat,
}
