mod customer;
mod extra_service;
mod media;
mod prostitute;
mod reservation;
mod schedule;
mod service;

use std::fmt;

use derive_more::{Display, From, FromStr};
use num_format::Locale;
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub use self::customer::*;
pub use self::extra_service::*;
pub use self::media::*;
pub use self::prostitute::*;
pub use self::reservation::*;
pub use self::schedule::*;
pub use self::service::*;

/// コアイベント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum CoreEvent {
    /// オプションサービスイベント
    ExtraServiceEvent(ExtraServiceEvent),
    /// メディアイベント
    MediaEvent(MediaEvent),
    /// 女の子イベント
    ProstituteEvent(ProstituteEvent),
    /// スケジュールイベント
    ScheduleEvent(ScheduleEvent),
}

/// MIMEタイプ
#[serde_as]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, From, FromStr, Display)]
pub struct Mime(#[serde_as(as = "DisplayFromStr")] mime::Mime);

impl Mime {
    const IMAGE_JPEG: Mime = Self(mime::IMAGE_JPEG);
    const IMAGE_GIF: Mime = Self(mime::IMAGE_GIF);
    const IMAGE_PNG: Mime = Self(mime::IMAGE_PNG);
}

impl Default for Mime {
    fn default() -> Self {
        Self(mime::STAR_STAR)
    }
}

/// 金額
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Money {
    amount: i64,
    currency: Currency,
}

impl Money {
    pub fn new(amount: i64, currency: Currency) -> Money {
        Money { amount, currency }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.currency {
            Currency::JPY => write!(f, "¥{}", self.amount.to_formatted_string(&Locale::ja)),
        }
    }
}

/// 通貨
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Currency {
    /// 日本円
    #[default]
    JPY,
}

/// 価格単位
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PriceUnit {
    /// 一回限りの価格
    #[default]
    OneTime,
    /// 時間単位の価格
    Hourly,
}

/// 価格
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Price {
    amount: Money,
    unit: PriceUnit,
}

impl Price {
    pub fn new(amount: Money, unit: PriceUnit) -> Price {
        Price { amount, unit }
    }
}
