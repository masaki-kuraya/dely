mod extra_service;
mod media;
mod prostitute;
mod schedule;

use derive_more::{Display, From, FromStr};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub use self::extra_service::*;
pub use self::media::*;
pub use self::prostitute::*;
pub use self::schedule::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum CoreEvent {
    ExtraServiceEvent(ExtraServiceEvent),
    MediaEvent(MediaEvent),
    ProstituteEvent(ProstituteEvent),
    ScheduleEvent(ScheduleEvent),
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, From, FromStr, Display)]
pub struct Mime(#[serde_as(as = "DisplayFromStr")] mime::Mime);

impl Default for Mime {
    fn default() -> Self {
        Self(mime::STAR_STAR)
    }
}
