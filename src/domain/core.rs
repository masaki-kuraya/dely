mod extra_service;
mod media;
mod prostitute;
mod shift;

use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

pub use self::extra_service::*;
pub use self::media::*;
pub use self::prostitute::*;
pub use self::shift::*;

pub enum CoreEvent {
    ProstituteEvent(ProstituteEvent),
    ExtraServiceEvent(ExtraServiceEvent),
}

#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mime(#[serde_as(as = "DisplayFromStr")] mime::Mime);

impl From<mime::Mime> for Mime {
    fn from(value: mime::Mime) -> Self {
        Self(value)
    }
}

impl FromStr for Mime {
    type Err = mime::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        mime::Mime::from_str(s).map(Mime::from)
    }
}

impl Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Default for Mime {
    fn default() -> Self {
        Self(mime::STAR_STAR)
    }
}
