use super::MediaId;
use crate::domain::{DataAccessError, Entity, Event, EventQueue, EventQueueIntoIter, Id};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, ops::Deref};

#[async_trait]
pub trait ProstituteRepository {
    async fn find_by_id(&self, id: ProstituteId) -> Result<Option<Prostitute>, DataAccessError>;
    async fn save(&mut self, entity: &mut Prostitute) -> Result<bool, DataAccessError>;
    async fn delete(&mut self, entity: &mut Prostitute) -> Result<bool, DataAccessError>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProstituteId(u64);

impl Id for ProstituteId {
    type Inner = u64;
}

impl fmt::Display for ProstituteId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for ProstituteId {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for ProstituteId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProstituteEvent {
    Joined {
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
    Rejoined {
        id: ProstituteId,
    },
    Leaved {
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
    QuestionRemoved {
        id: ProstituteId,
        index: usize,
    },
    QuestionOrderChanged {
        id: ProstituteId,
        index_a: usize,
        index_b: usize,
    },
    ImagesChanged {
        id: ProstituteId,
        images: Vec<MediaId>,
    },
    ImageAdded {
        id: ProstituteId,
        image: MediaId,
    },
    ImageRemoved {
        id: ProstituteId,
        image: MediaId,
    },
    ImageOrderChanged {
        id: ProstituteId,
        image_a: MediaId,
        image_b: MediaId,
    },
    VideoChanged {
        id: ProstituteId,
        video: Option<MediaId>,
    },
}

impl Event for ProstituteEvent {
    type Id = ProstituteId;
}

#[derive(Clone, Default, Debug)]
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
        let mut entity = Prostitute::default();
        let event = ProstituteEvent::Joined {
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
        };
        entity.validate(&event)?;
        entity.apply(event);
        Ok(entity)
    }

    pub fn change_name(&mut self, name: String) -> Result<(), ProstituteError> {
        let event = ProstituteEvent::NameChanged { id: self.id, name };
        self.validate(&event)?;
        self.apply(event);
        Ok(())
    }

    pub fn change_catchphrase(&mut self, catchphrase: String) -> Result<(), ProstituteError> {
        let event = ProstituteEvent::CatchphraseChanged {
            id: self.id,
            catchphrase,
        };
        self.validate(&event)?;
        self.apply(event);
        Ok(())
    }

    pub fn change_profile(&mut self, profile: String) {
        let event = ProstituteEvent::ProfileChanged {
            id: self.id,
            profile,
        };
        self.apply(event);
    }

    pub fn change_message(&mut self, message: String) {
        let event = ProstituteEvent::MessageChanged {
            id: self.id,
            message,
        };
        self.apply(event);
    }

    pub fn change_figure(&mut self, figure: Figure) {
        let event = ProstituteEvent::FigureChanged {
            id: self.id,
            figure,
        };
        self.apply(event);
    }

    pub fn change_blood_type(&mut self, blood: Option<BloodType>) {
        let event = ProstituteEvent::BloodTypeChanged { id: self.id, blood };
        self.apply(event);
    }

    pub fn change_birthday(&mut self, birthday: Option<Birthday>) {
        let event = ProstituteEvent::BirthdayChanged {
            id: self.id,
            birthday,
        };
        self.apply(event);
    }

    pub fn add_question(&mut self, question: Question) {
        let event = ProstituteEvent::QuestionAdded {
            id: self.id,
            question,
        };
        self.apply(event);
    }

    pub fn remove_question(&mut self, index: usize) -> Result<(), ProstituteError> {
        let event = ProstituteEvent::QuestionRemoved { id: self.id, index };
        self.validate(&event)?;
        self.apply(event);
        Ok(())
    }

    pub fn change_question_order(
        &mut self,
        index_a: usize,
        index_b: usize,
    ) -> Result<(), ProstituteError> {
        let event = ProstituteEvent::QuestionOrderChanged {
            id: self.id,
            index_a,
            index_b,
        };
        self.validate(&event)?;
        self.apply(event);
        Ok(())
    }

    pub fn change_questions(&mut self, questions: Vec<Question>) {
        let event = ProstituteEvent::QuestionsChanged {
            id: self.id,
            questions,
        };
        self.apply(event);
    }

    pub fn change_images(&mut self, images: Vec<MediaId>) {
        let event = ProstituteEvent::ImagesChanged {
            id: self.id,
            images,
        };
        self.apply(event);
    }

    pub fn change_video(&mut self, video: Option<MediaId>) {
        let event = ProstituteEvent::VideoChanged { id: self.id, video };
        self.apply(event);
        self.video = video;
    }
}

impl Entity for Prostitute {
    type Id = ProstituteId;
    type Event = ProstituteEvent;
    type Error = ProstituteError;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        let validate_name = |name: &str| {
            if name.trim().is_empty() {
                Err(ProstituteError::NameIsEmpty)
            } else {
                Ok(())
            }
        };
        let validate_catchphrase = |catchphrase: &str| {
            if catchphrase.trim().is_empty() {
                Err(ProstituteError::CatchphraseIsEmpty)
            } else {
                Ok(())
            }
        };
        match event {
            ProstituteEvent::Joined {
                name, catchphrase, ..
            } => {
                validate_name(name)?;
                validate_catchphrase(catchphrase)
            }
            ProstituteEvent::Rejoined { .. } => Ok(()),
            ProstituteEvent::Leaved { .. } => Ok(()),
            ProstituteEvent::NameChanged { name, .. } => validate_name(name),
            ProstituteEvent::CatchphraseChanged { catchphrase, .. } => {
                validate_catchphrase(catchphrase)
            }
            ProstituteEvent::ProfileChanged { .. } => Ok(()),
            ProstituteEvent::MessageChanged { .. } => Ok(()),
            ProstituteEvent::FigureChanged { .. } => Ok(()),
            ProstituteEvent::BloodTypeChanged { .. } => Ok(()),
            ProstituteEvent::BirthdayChanged { .. } => Ok(()),
            ProstituteEvent::QuestionsChanged { .. } => Ok(()),
            ProstituteEvent::QuestionAdded { .. } => Ok(()),
            ProstituteEvent::QuestionRemoved { index, .. } => {
                if index >= &self.questions.len() {
                    Err(ProstituteError::QuestionIsNotExists)
                } else {
                    Ok(())
                }
            }
            ProstituteEvent::QuestionOrderChanged {
                index_a, index_b, ..
            } => {
                if index_a >= &self.questions.len() {
                    Err(ProstituteError::QuestionIsNotExists)
                } else if index_b >= &self.questions.len() {
                    Err(ProstituteError::QuestionIsNotExists)
                } else {
                    Ok(())
                }
            }
            ProstituteEvent::ImagesChanged { .. } => Ok(()),
            ProstituteEvent::ImageAdded { .. } => Ok(()),
            ProstituteEvent::ImageRemoved { .. } => Ok(()),
            ProstituteEvent::ImageOrderChanged { .. } => Ok(()),
            ProstituteEvent::VideoChanged { .. } => Ok(()),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        if let Err(_) = self.validate(&event) {
            return;
        }
        match event.clone() {
            ProstituteEvent::Joined {
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
                    return;
                }
                self.id = id;
                self.name = name;
                self.catchphrase = catchphrase;
                self.profile = profile;
                self.message = message;
                self.figure = figure;
                self.blood = blood;
                self.birthday = birthday;
                self.questions = questions;
                self.images = images;
                self.video = video;
            }
            ProstituteEvent::Rejoined { id } => {
                if self.id != id || self.leaved {
                    return;
                }
                self.leaved = false;
            }
            ProstituteEvent::Leaved { id } => {
                if self.id != id || !self.leaved {
                    return;
                }
                self.leaved = true;
            }
            ProstituteEvent::NameChanged { id, name } => {
                if self.id != id {
                    return;
                }
                self.name = name;
            }
            ProstituteEvent::CatchphraseChanged { id, catchphrase } => {
                if self.id != id {
                    return;
                }
                self.catchphrase = catchphrase;
            }
            ProstituteEvent::ProfileChanged { id, profile } => {
                if self.id != id {
                    return;
                }
                self.profile = profile;
            }
            ProstituteEvent::MessageChanged { id, message } => {
                if self.id != id {
                    return;
                }
                self.message = message;
            }
            ProstituteEvent::FigureChanged { id, figure } => {
                if self.id != id {
                    return;
                }
                self.figure = figure;
            }
            ProstituteEvent::BloodTypeChanged { id, blood } => {
                if self.id != id {
                    return;
                }
                self.blood = blood;
            }
            ProstituteEvent::BirthdayChanged { id, birthday } => {
                if self.id != id {
                    return;
                }
                self.birthday = birthday;
            }
            ProstituteEvent::QuestionsChanged { id, questions } => {
                if self.id != id {
                    return;
                }
                self.questions = questions;
            }
            ProstituteEvent::QuestionAdded { id, question } => {
                if self.id != id {
                    return;
                }
                self.questions.push(question);
            }
            ProstituteEvent::QuestionRemoved { id, index } => {
                if self.id != id {
                    return;
                }
                self.questions.remove(index);
            }
            ProstituteEvent::QuestionOrderChanged {
                id,
                index_a,
                index_b,
            } => {
                if self.id != id {
                    return;
                }
                self.questions.swap(index_a, index_b);
            }
            ProstituteEvent::ImagesChanged { id, images } => {
                if self.id != id {
                    return;
                }
                self.images = images;
            }
            ProstituteEvent::ImageAdded { id, image } => {
                if self.id != id {
                    return;
                }
                self.images.push(image)
            }
            ProstituteEvent::ImageRemoved { id, image } => {
                if self.id != id {
                    return;
                }
                if let Some(pos) = self.images.iter().position(|x| *x == image) {
                    self.images.remove(pos);
                }
                return;
            }
            ProstituteEvent::ImageOrderChanged {
                id,
                image_a,
                image_b,
            } => {
                if self.id != id {
                    return;
                }
                let swaps: Vec<_> = self
                    .images
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        if *x == image_a || *x == image_b {
                            Some(i)
                        } else {
                            None
                        }
                    })
                    .collect();
                if swaps.len() == 2 {
                    self.images.swap(swaps[0], swaps[1]);
                }
                return;
            }
            ProstituteEvent::VideoChanged { id, video } => {
                if self.id != id {
                    return;
                }
                self.video = video;
            }
        }
        self.events.push(event);
    }

    fn entity_name() -> &'static str {
        "prostitute"
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl IntoIterator for Prostitute {
    type Item = ProstituteEvent;
    type IntoIter = EventQueueIntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}

#[derive(Debug)]
pub enum ProstituteError {
    NameIsEmpty,
    CatchphraseIsEmpty,
    QuestionIsNotExists,
}

impl fmt::Display for ProstituteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameIsEmpty => f.write_str("Name cannot be empty"),
            Self::CatchphraseIsEmpty => f.write_str("Name cannot be empty"),
            Self::QuestionIsNotExists => f.write_str("Question is not exists"),
        }
    }
}

impl Error for ProstituteError {}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Figure {
    pub vital_statistics: Option<VitalStatistics>,
    pub tall: Option<u16>,
    pub weight: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VitalStatistics {
    pub waist: u16,
    pub bust: Bust,
    pub hip: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bust {
    pub top: u16,
    pub under: Option<u16>,
}

#[derive(Debug, PartialEq, Eq)]
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

impl Bust {
    pub fn cup_size(&self) -> Option<CupSize> {
        Some(CupSize::from(self.top, self.under?))
    }
}

impl CupSize {
    pub fn from(top_bust: u16, under_bust: u16) -> Self {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
        assert_eq!(CupSize::from(90, 65), CupSize::G);
    }
}
