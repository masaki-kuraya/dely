use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use derive_more::{Deref, Display, Error, From, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

use super::MediaId;

#[async_trait]
pub trait ProstituteRepository {
    async fn find_by_id(&self, id: ProstituteId) -> Result<Option<Prostitute>, DataAccessError>;
    async fn save(&mut self, entity: &mut Prostitute) -> Result<bool, DataAccessError>;
    async fn delete(&mut self, entity: &mut Prostitute) -> Result<bool, DataAccessError>;
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ProstituteId(u64);

impl Id for ProstituteId {
    type Inner = u64;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProstituteEvent {
    ProstituteJoined {
        id: ProstituteId,
        name: String,
        catchphrase: String,
        profile: String,
        message: String,
        figure: Figure,
        blood: Option<BloodType>,
        birthday: Option<Birthday>,
        questions: Vec<Question>,
        images: Vec<MediaId>,
        video: Option<MediaId>,
    },
    ProstituteRejoined {
        id: ProstituteId,
    },
    ProstituteLeaved {
        id: ProstituteId,
    },
    NameChanged {
        id: ProstituteId,
        name: String,
    },
    CatchphraseChanged {
        id: ProstituteId,
        catchphrase: String,
    },
    ProfileChanged {
        id: ProstituteId,
        profile: String,
    },
    MessageChanged {
        id: ProstituteId,
        message: String,
    },
    FigureChanged {
        id: ProstituteId,
        figure: Figure,
    },
    BloodTypeChanged {
        id: ProstituteId,
        blood: Option<BloodType>,
    },
    BirthdayChanged {
        id: ProstituteId,
        birthday: Option<Birthday>,
    },
    QuestionsChanged {
        id: ProstituteId,
        questions: Vec<Question>,
    },
    QuestionAdded {
        id: ProstituteId,
        question: Question,
    },
    QuestionDeleted {
        id: ProstituteId,
        index: usize,
    },
    QuestionSwapped {
        id: ProstituteId,
        index_a: usize,
        index_b: usize,
    },
    ImagesChanged {
        id: ProstituteId,
        media_ids: Vec<MediaId>,
    },
    ImageAdded {
        id: ProstituteId,
        media_id: MediaId,
    },
    ImageDeleted {
        id: ProstituteId,
        media_id: MediaId,
    },
    ImageSwapped {
        id: ProstituteId,
        media_id_a: MediaId,
        media_id_b: MediaId,
    },
    VideoChanged {
        id: ProstituteId,
        media_id: Option<MediaId>,
    },
    ProstituteDeleted {
        id: ProstituteId,
    },
}

impl Event for ProstituteEvent {
    type Id = ProstituteId;
}

#[derive(Clone, Default, Debug, IntoIterator, Serialize, Deserialize)]
pub struct Prostitute {
    id: ProstituteId,
    name: String,
    catchphrase: String,
    profile: String,
    message: String,
    figure: Figure,
    blood: Option<BloodType>,
    birthday: Option<Birthday>,
    questions: Vec<Question>,
    images: Vec<MediaId>,
    video: Option<MediaId>,
    leaved: bool,
    #[serde(skip)]
    #[into_iterator]
    events: EventQueue<ProstituteEvent>,
}

impl Prostitute {
    pub fn join(
        id: ProstituteId,
        name: String,
        catchphrase: String,
        profile: String,
        message: String,
        figure: Figure,
        blood: Option<BloodType>,
        birthday: Option<Birthday>,
        questions: Vec<Question>,
        images: Vec<MediaId>,
        video: Option<MediaId>,
    ) -> Result<Self, ProstituteError> {
        Self::validate_created(&name, &catchphrase, &images)?;
        let mut entity = Prostitute {
            id,
            name: name.clone(),
            catchphrase: catchphrase.clone(),
            profile: profile.clone(),
            message: message.clone(),
            figure: figure.clone(),
            blood,
            birthday: birthday.clone(),
            questions: questions.clone(),
            images: images.clone(),
            video,
            ..Default::default()
        };
        entity.events.push(ProstituteEvent::ProstituteJoined {
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
        });
        Ok(entity)
    }

    pub fn rejoin(&mut self) -> Result<(), ProstituteError> {
        self.validate_rejoined()?;
        self.leaved = false;
        self.events
            .push(ProstituteEvent::ProstituteLeaved { id: self.id });
        Ok(())
    }

    pub fn leave(&mut self) -> Result<(), ProstituteError> {
        self.validate_leaved()?;
        self.leaved = true;
        self.events
            .push(ProstituteEvent::ProstituteLeaved { id: self.id });
        Ok(())
    }

    pub fn change_name(&mut self, name: String) -> Result<(), ProstituteError> {
        Self::validate_name(&name)?;
        self.name = name.clone();
        self.events
            .push(ProstituteEvent::NameChanged { id: self.id, name });
        Ok(())
    }

    pub fn change_catchphrase(&mut self, catchphrase: String) -> Result<(), ProstituteError> {
        Self::validate_catchphrase(&catchphrase)?;
        self.catchphrase = catchphrase.clone();
        self.events.push(ProstituteEvent::CatchphraseChanged {
            id: self.id,
            catchphrase,
        });
        Ok(())
    }

    pub fn change_profile(&mut self, profile: String) {
        self.profile = profile.clone();
        self.events.push(ProstituteEvent::ProfileChanged {
            id: self.id,
            profile,
        })
    }

    pub fn change_message(&mut self, message: String) {
        self.message = message.clone();
        self.events.push(ProstituteEvent::MessageChanged {
            id: self.id,
            message,
        })
    }

    pub fn change_figure(&mut self, figure: Figure) {
        self.figure = figure.clone();
        self.events.push(ProstituteEvent::FigureChanged {
            id: self.id,
            figure,
        })
    }

    pub fn change_blood_type(&mut self, blood: Option<BloodType>) {
        self.blood = blood;
        self.events
            .push(ProstituteEvent::BloodTypeChanged { id: self.id, blood })
    }

    pub fn change_birthday(&mut self, birthday: Option<Birthday>) {
        self.birthday = birthday.clone();
        self.events.push(ProstituteEvent::BirthdayChanged {
            id: self.id,
            birthday,
        })
    }

    pub fn change_questions(&mut self, questions: Vec<Question>) {
        self.questions = questions.clone();
        self.events.push(ProstituteEvent::QuestionsChanged {
            id: self.id,
            questions,
        })
    }

    pub fn add_question(&mut self, question: Question) {
        self.questions.push(question.clone());
        self.events.push(ProstituteEvent::QuestionAdded {
            id: self.id,
            question,
        })
    }

    pub fn delete_question(&mut self, index: usize) -> Result<(), ProstituteError> {
        self.validate_question_delete(&index)?;
        self.questions.remove(index);
        self.events
            .push(ProstituteEvent::QuestionDeleted { id: self.id, index });
        Ok(())
    }

    pub fn swap_question(&mut self, index_a: usize, index_b: usize) -> Result<(), ProstituteError> {
        self.validate_question_swapped(&index_a, &index_b)?;
        self.questions.swap(index_a, index_b);
        self.events.push(ProstituteEvent::QuestionSwapped {
            id: self.id,
            index_a,
            index_b,
        });
        Ok(())
    }

    pub fn change_images(&mut self, media_ids: Vec<MediaId>) {
        self.images = media_ids.clone();
        self.events.push(ProstituteEvent::ImagesChanged {
            id: self.id,
            media_ids,
        });
    }

    pub fn add_image(&mut self, media_id: MediaId) -> Result<(), ProstituteError> {
        self.validate_image_added(&media_id)?;
        self.images.push(media_id);
        self.events.push(ProstituteEvent::ImageAdded {
            id: self.id,
            media_id,
        });
        Ok(())
    }

    pub fn delete_image(&mut self, media_id: MediaId) -> Result<(), ProstituteError> {
        self.validate_image_deleted(&media_id)?;
        self.images.retain(|&m| m != media_id);
        self.events.push(ProstituteEvent::ImageDeleted {
            id: self.id,
            media_id,
        });
        Ok(())
    }

    pub fn swap_image(
        &mut self,
        media_id_a: MediaId,
        media_id_b: MediaId,
    ) -> Result<(), ProstituteError> {
        self.validate_image_swapped(&media_id_a, &media_id_b)?;
        self.images.iter_mut().for_each(|x| {
            if *x == media_id_a {
                *x = media_id_b
            } else if *x == media_id_b {
                *x = media_id_a
            }
        });
        self.events.push(ProstituteEvent::ImageSwapped {
            id: self.id,
            media_id_a,
            media_id_b,
        });
        Ok(())
    }

    pub fn change_video(&mut self, video: Option<MediaId>) {
        let event = ProstituteEvent::VideoChanged {
            id: self.id,
            media_id: video,
        };
        self.apply(event);
        self.video = video;
    }

    fn validate_id(&self, id: &ProstituteId) -> Result<(), ProstituteError> {
        match self.id == *id {
            true => Ok(()),
            false => Err(ProstituteError::MismatchedId),
        }
    }

    fn validate_created(
        name: &String,
        catchphrase: &String,
        images: &Vec<MediaId>,
    ) -> Result<(), ProstituteError> {
        Self::validate_name(name)?;
        Self::validate_catchphrase(catchphrase)?;
        let new_images: HashSet<_> = HashSet::from_iter(images);
        match images.len() == new_images.len() {
            true => Ok(()),
            false => Err(ProstituteError::DuplicateImage),
        }
    }

    fn validate_rejoined(&self) -> Result<(), ProstituteError> {
        match self.leaved {
            true => Err(ProstituteError::AlreadyLeft),
            false => Ok(()),
        }
    }

    fn validate_leaved(&self) -> Result<(), ProstituteError> {
        match self.leaved {
            true => Ok(()),
            false => Err(ProstituteError::AlreadyJoined),
        }
    }

    fn validate_name(name: &str) -> Result<(), ProstituteError> {
        match name.trim().is_empty() {
            true => Err(ProstituteError::NameIsBlank),
            false => Ok(()),
        }
    }

    fn validate_catchphrase(catchphrase: &str) -> Result<(), ProstituteError> {
        match catchphrase.trim().is_empty() {
            true => Err(ProstituteError::CatchphraseIsBlank),
            false => Ok(()),
        }
    }

    fn validate_question_delete(&self, index: &usize) -> Result<(), ProstituteError> {
        self.validate_question_not_found(index)
    }

    fn validate_question_swapped(
        &self,
        index_a: &usize,
        index_b: &usize,
    ) -> Result<(), ProstituteError> {
        self.validate_question_not_found(index_a)?;
        self.validate_question_not_found(index_b)?;
        match *index_a == *index_b {
            true => Err(ProstituteError::DuplicateQuestionIndex),
            false => Ok(()),
        }
    }

    fn validate_image_added(&self, media_id: &MediaId) -> Result<(), ProstituteError> {
        match self.images.iter().find(|&&id| id == *media_id) {
            Some(_) => Err(ProstituteError::DuplicateImage),
            None => Ok(()),
        }
    }

    fn validate_image_deleted(&self, media_id: &MediaId) -> Result<(), ProstituteError> {
        self.validate_image_not_found(media_id)
    }

    fn validate_image_swapped(
        &self,
        media_id_a: &MediaId,
        media_id_b: &MediaId,
    ) -> Result<(), ProstituteError> {
        self.validate_image_not_found(media_id_a)?;
        self.validate_image_not_found(media_id_b)?;
        match *media_id_a == *media_id_b {
            true => Err(ProstituteError::DuplicateImageIndex),
            false => Ok(()),
        }
    }

    fn validate_question_not_found(&self, index: &usize) -> Result<(), ProstituteError> {
        if *index >= self.questions.len() {
            Err(ProstituteError::QuestionNotFound)
        } else {
            Ok(())
        }
    }

    fn validate_image_not_found(&self, media_id: &MediaId) -> Result<(), ProstituteError> {
        match self.images.iter().find(|&&id| id == *media_id) {
            Some(_) => Ok(()),
            None => Err(ProstituteError::ImageNotFound),
        }
    }
}

impl Entity for Prostitute {
    type Id = ProstituteId;

    const ENTITY_NAME: &'static str = "prostitute";

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Aggregation for Prostitute {
    type Event = ProstituteEvent;
    type Error = ProstituteError;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            ProstituteEvent::ProstituteJoined {
                name,
                catchphrase,
                images,
                ..
            } => Self::validate_created(name, catchphrase, images),
            ProstituteEvent::ProstituteRejoined { id } => {
                self.validate_id(id)?;
                self.validate_rejoined()
            }
            ProstituteEvent::ProstituteLeaved { id } => {
                self.validate_id(id)?;
                self.validate_leaved()
            }
            ProstituteEvent::NameChanged { id, name } => {
                self.validate_id(id)?;
                Self::validate_name(name)
            }
            ProstituteEvent::CatchphraseChanged { id, catchphrase } => {
                self.validate_id(id)?;
                Self::validate_catchphrase(catchphrase)
            }
            ProstituteEvent::ProfileChanged { id, .. }
            | ProstituteEvent::MessageChanged { id, .. }
            | ProstituteEvent::FigureChanged { id, .. }
            | ProstituteEvent::BloodTypeChanged { id, .. }
            | ProstituteEvent::BirthdayChanged { id, .. }
            | ProstituteEvent::QuestionsChanged { id, .. }
            | ProstituteEvent::QuestionAdded { id, .. } => self.validate_id(id),
            ProstituteEvent::QuestionDeleted { id, index } => {
                self.validate_id(id)?;
                self.validate_question_delete(index)
            }
            ProstituteEvent::QuestionSwapped {
                id,
                index_a,
                index_b,
            } => {
                self.validate_id(id)?;
                self.validate_question_swapped(index_a, index_b)
            }
            ProstituteEvent::ImagesChanged { id, .. } => self.validate_id(id),
            ProstituteEvent::ImageAdded { id, media_id } => {
                self.validate_id(id)?;
                self.validate_image_added(media_id)
            }
            ProstituteEvent::ImageDeleted { id, media_id } => {
                self.validate_id(id)?;
                self.validate_image_deleted(media_id)
            }
            ProstituteEvent::ImageSwapped {
                id,
                media_id_a,
                media_id_b,
            } => {
                self.validate_id(id)?;
                self.validate_image_swapped(media_id_a, media_id_b)
            }
            ProstituteEvent::VideoChanged { id, .. }
            | ProstituteEvent::ProstituteDeleted { id, .. } => self.validate_id(id),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
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
                if self.id == id {
                    if let Ok(entity) = Self::join(
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
                        *self = entity;
                    }
                }
            }
            ProstituteEvent::ProstituteRejoined { id } => {
                if self.id == id {
                    if let Err(_e) = self.rejoin() {}
                }
            }
            ProstituteEvent::ProstituteLeaved { id } => {
                if self.id == id {
                    if let Err(_e) = self.leave() {}
                }
            }
            ProstituteEvent::NameChanged { id, name } => {
                if self.id == id {
                    if let Err(_e) = self.change_name(name) {}
                }
            }
            ProstituteEvent::CatchphraseChanged { id, catchphrase } => {
                if self.id == id {
                    if let Err(_e) = self.change_catchphrase(catchphrase) {}
                }
            }
            ProstituteEvent::ProfileChanged { id, profile } => {
                if self.id == id {
                    self.change_profile(profile);
                }
            }
            ProstituteEvent::MessageChanged { id, message } => {
                if self.id == id {
                    self.change_message(message)
                }
            }
            ProstituteEvent::FigureChanged { id, figure } => {
                if self.id == id {
                    self.change_figure(figure)
                }
            }
            ProstituteEvent::BloodTypeChanged { id, blood } => {
                if self.id == id {
                    self.change_blood_type(blood)
                }
            }
            ProstituteEvent::BirthdayChanged { id, birthday } => {
                if self.id == id {
                    self.change_birthday(birthday)
                }
            }
            ProstituteEvent::QuestionsChanged { id, questions } => {
                if self.id == id {
                    self.change_questions(questions)
                }
            }
            ProstituteEvent::QuestionAdded { id, question } => {
                if self.id == id {
                    self.add_question(question)
                }
            }
            ProstituteEvent::QuestionDeleted { id, index } => {
                if self.id == id {
                    if let Err(_e) = self.delete_question(index) {}
                }
            }
            ProstituteEvent::QuestionSwapped {
                id,
                index_a,
                index_b,
            } => {
                if self.id == id {
                    if let Err(_e) = self.swap_question(index_a, index_b) {}
                }
            }
            ProstituteEvent::ImagesChanged { id, media_ids } => {
                if self.id == id {
                    self.change_images(media_ids)
                }
            }
            ProstituteEvent::ImageAdded { id, media_id } => {
                if self.id == id {
                    if let Err(_e) = self.add_image(media_id) {}
                }
            }
            ProstituteEvent::ImageDeleted { id, media_id } => {
                if self.id == id {
                    if let Err(_e) = self.delete_image(media_id) {}
                }
            }
            ProstituteEvent::ImageSwapped {
                id,
                media_id_a,
                media_id_b,
            } => {
                if self.id == id {
                    if let Err(_e) = self.swap_image(media_id_a, media_id_b) {}
                }
            }
            ProstituteEvent::VideoChanged { id, media_id } => {
                if self.id == id {
                    self.change_video(media_id)
                }
            }
            ProstituteEvent::ProstituteDeleted { .. } => {}
        }
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl PartialEq for Prostitute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.name == other.name
            && self.catchphrase == other.catchphrase
            && self.profile == other.profile
            && self.message == other.message
            && self.figure == other.figure
            && self.blood == other.blood
            && self.birthday == other.birthday
            && self.questions == other.questions
            && self.images == other.images
            && self.video == other.video
            && self.leaved == other.leaved
    }
}

impl Eq for Prostitute {}

#[derive(Error, Display, Debug)]
pub enum ProstituteError {
    /// IDが一致しません
    #[display(fmt = "ID does not match")]
    MismatchedId,
    /// 既に退店しています          
    #[display(fmt = "This prostitute has already left")]
    AlreadyLeft,
    /// 既に入店しています
    #[display(fmt = "This prostitute has already joined")]
    AlreadyJoined,
    /// 名前が空欄です                  
    #[display(fmt = "Name cannot be blank")]
    NameIsBlank,
    /// キャッチフレーズが空欄です
    #[display(fmt = "Catchphrase cannot be blank")]
    CatchphraseIsBlank,
    /// 質問が見つかりません
    #[display(fmt = "Question not found")]
    QuestionNotFound,
    /// 質問のインデックスが重複しています
    #[display(fmt = "Duplicate question index")]
    DuplicateQuestionIndex,
    /// 画像が既に存在します
    #[display(fmt = "Image already exists")]
    DuplicateImage,
    /// 画像が見つかりません
    #[display(fmt = "Image not found")]
    ImageNotFound,
    /// 画像のインデックスが重複しています
    #[display(fmt = "Duplicate image index")]
    DuplicateImageIndex,
}

/// 体型
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Figure {
    pub vital_statistics: Option<VitalStatistics>,
    pub cup_size: Option<CupSize>,
    pub height: Option<u16>,
    pub weight: Option<u16>,
}

impl Figure {
    pub fn bmi(&self) -> Option<f32> {
        let fn_bmi = |(weight, height): (f32, f32)| weight / height.powi(2);
        self.weight
            .map(f32::from)
            .zip(self.height.map(f32::from))
            .map(fn_bmi)
    }

    pub fn cup_size(&self) -> Option<CupSize> {
        self.cup_size.or(self
            .vital_statistics
            .clone()
            .map_or(None, |v| v.bust.cup_size()))
    }

    pub fn figure_type(&self) -> Vec<FigureType> {
        let mut result = Vec::new();
        if let Some(h) = self.height {
            if h < 155 {
                result.push(FigureType::Petite);
            }
            if h >= 165 {
                result.push(FigureType::Tall);
            }
            if let Some(v) = &self.vital_statistics {
                if v.bust.top > h * 58 / 100 && v.waist < h * 41 / 100 && v.hip > h * 58 / 100 {
                    result.push(FigureType::Voluptuous);
                }
            }
        }
        if let Some(bmi) = self.bmi() {
            if bmi < 18.5 {
                result.push(FigureType::Slender);
            }
            if bmi >= 23.0 && bmi < 25.0 {
                result.push(FigureType::Plump);
            }
            if bmi >= 25.0 {
                result.push(FigureType::Chubby);
            }
        }
        if self.height.is_some() && self.weight.is_some() && result.is_empty() {
            result.push(FigureType::Normal);
        }
        result
    }
}

/// 体型の種類
#[derive(Debug, Display, PartialEq, Eq)]
pub enum FigureType {
    /// スレンダー
    #[display(fmt = "スレンダー")]
    Slender,
    /// ぽっちゃり
    #[display(fmt = "ぽっちゃり")]
    Plump,
    /// 小柄
    #[display(fmt = "小柄")]
    Petite,
    /// 長身
    #[display(fmt = "長身")]
    Tall,
    /// 普通
    #[display(fmt = "普通")]
    Normal,
    /// グラマー
    #[display(fmt = "グラマー")]
    Voluptuous,
    /// 肥満
    #[display(fmt = "肥満")]
    Chubby,
}

/// スリーサイズ
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VitalStatistics {
    pub waist: u16,
    pub bust: Bust,
    pub hip: u16,
}

/// バスト
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bust {
    pub top: u16,
    pub under: Option<u16>,
}

impl Bust {
    pub fn cup_size(&self) -> Option<CupSize> {
        Some(CupSize::new(self.top, self.under?))
    }
}

/// カップサイズ
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum CupSize {
    AAA,
    AA,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl CupSize {
    pub fn new(top_bust: u16, under_bust: u16) -> Self {
        match top_bust.checked_sub(under_bust) {
            None => Self::AAA,
            Some(0..=6) => Self::AAA,
            Some(7..=8) => Self::AA,
            Some(9..=11) => Self::A,
            Some(12..=13) => Self::B,
            Some(14..=16) => Self::C,
            Some(17..=18) => Self::D,
            Some(19..=21) => Self::E,
            Some(22..=23) => Self::F,
            Some(24..=26) => Self::G,
            Some(27..=28) => Self::H,
            Some(29..=31) => Self::I,
            Some(32..=33) => Self::J,
            Some(34..=36) => Self::K,
            Some(37..=38) => Self::L,
            Some(39..=41) => Self::M,
            Some(42..=43) => Self::N,
            Some(44..=46) => Self::O,
            Some(47..=48) => Self::P,
            Some(49..=51) => Self::Q,
            Some(52..=53) => Self::R,
            Some(54..=56) => Self::S,
            Some(57..=58) => Self::T,
            Some(59..=61) => Self::U,
            Some(62..=63) => Self::V,
            Some(64..=66) => Self::W,
            Some(67..=68) => Self::X,
            Some(69..=71) => Self::Y,
            Some(_) => Self::Z,
        }
    }
}

/// 血液型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum BloodType {
    A,
    B,
    O,
    AB,
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize)]
pub struct Birthday(NaiveDate);

impl Birthday {
    pub fn age<Tz: TimeZone>(&self, timezone: &Tz) -> i32 {
        let current = Utc::now().with_timezone(timezone);
        let birthday = DateTime::<Tz>::from_utc(
            NaiveDateTime::new(self.0, NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
            timezone.offset_from_utc_date(&self.0),
        );
        let age = current.year() - birthday.year();
        if current.month() > birthday.month() {
            age
        } else if current.month() == birthday.month() && current.day() >= birthday.day() {
            age
        } else {
            age - 1
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Question {
    pub question: String,
    pub answer: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cup_size_from() {
        assert_eq!(CupSize::new(88, 65), CupSize::F);
        assert_eq!(CupSize::new(89, 65), CupSize::G);
        assert_eq!(CupSize::new(91, 65), CupSize::G);
        assert_eq!(CupSize::new(92, 65), CupSize::H);
    }
}
