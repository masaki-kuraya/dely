use std::ops::Range;

use async_trait::async_trait;
use bio::data_structures::interval_tree::IntervalTree;
use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, Error, From, IntoIterator};
use serde::{Deserialize, Serialize};

use crate::domain::{Aggregation, DataAccessError, Entity, Event, EventQueue, Id};

use super::ProstituteId;

/// スケジュールのリポジトリトレイト
#[async_trait]
pub trait ScheduleRepository {
    /// IDからスケジュールを取得する
    async fn find_by_id(&self, id: ScheduleId) -> Result<Option<Schedule>, DataAccessError>;
    /// スケジュールを保存する
    async fn save(&mut self, entity: &mut Schedule) -> Result<bool, DataAccessError>;
    /// スケジュールを削除する
    async fn delete(&mut self, entity: &mut Schedule) -> Result<bool, DataAccessError>;
}

/// スケジュールのID
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default,
)]
pub struct ScheduleId(u64);

impl Id for ScheduleId {
    type Inner = u64;
}

/// スケジュールのイベント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleEvent {
    /// スケジュールが作成された
    ScheduleCreated {
        id: ScheduleId,
        prostitute_id: ProstituteId,
    },
    /// スケジュールが削除された
    ScheduleDeleted { id: ScheduleId },
    /// スケジュールにシフトが追加された
    ShiftAdded { id: ScheduleId, shift: Shift },
    /// シフトの時間範囲が変更された
    ShiftTimeChanged {
        shift_id: ShiftId,
        time: Range<DateTime<Utc>>,
    },
    /// シフトのステータスが変更された
    ShiftStatusChanged {
        shift_id: ShiftId,
        status: ShiftStatus,
    },
    /// シフトが削除された
    ShiftsDeleted { shift_ids: Vec<ShiftId> },
}

impl Event for ScheduleEvent {
    type Id = ScheduleId;
}

#[derive(Debug, Clone, Default, IntoIterator, Serialize, Deserialize)]
pub struct Schedule {
    id: ScheduleId,
    prostitute_id: ProstituteId,
    shifts: Vec<Shift>,
    #[serde(skip)]
    #[into_iterator]
    events: EventQueue<ScheduleEvent>,
}

impl Schedule {
    pub fn create(id: ScheduleId, prostitute_id: ProstituteId) -> Self {
        let mut entity = Self {
            id,
            prostitute_id,
            ..Default::default()
        };
        entity
            .events
            .push(ScheduleEvent::ScheduleCreated { id, prostitute_id });
        entity
    }

    pub fn add_shift(&mut self, shift: Shift) -> Result<(), ScheduleError> {
        self.validate_shift_added(&shift)?;
        self.shifts.push(shift.clone());
        self.events
            .push(ScheduleEvent::ShiftAdded { id: self.id, shift });
        Ok(())
    }

    pub fn change_shift_time(
        &mut self,
        shift_id: ShiftId,
        time: Range<DateTime<Utc>>,
    ) -> Result<(), ScheduleError> {
        self.validate_shift_time_changed(&shift_id, &time)?;
        self.shifts
            .iter_mut()
            .filter(|s| s.id == shift_id)
            .for_each(|s| if let Err(_) = s.change_time(time.clone()) {});
        self.events
            .push(ScheduleEvent::ShiftTimeChanged { shift_id, time });
        Ok(())
    }

    pub fn change_shift_status(
        &mut self,
        shift_id: ShiftId,
        status: ShiftStatus,
    ) -> Result<(), ScheduleError> {
        self.validate_shift_status_changed(&shift_id, &status)?;
        self.shifts
            .iter_mut()
            .filter(|s| s.id == shift_id)
            .for_each(|s| if let Err(_) = s.change_status(status) {});
        self.events
            .push(ScheduleEvent::ShiftStatusChanged { shift_id, status });
        Ok(())
    }

    pub fn delete_shifts<T: IntoIterator<Item = ShiftId>>(
        &mut self,
        shift_ids: T,
    ) -> Result<(), ScheduleError> {
        let shift_ids = shift_ids.into_iter().collect::<Vec<_>>();
        self.validate_shifts_deleted(&shift_ids)?;
        self.shifts.retain(|shift| shift_ids.contains(&shift.id));
        self.events.push(ScheduleEvent::ShiftsDeleted { shift_ids });
        Ok(())
    }

    pub fn shift(&self, shift_id: &ShiftId) -> Option<&Shift> {
        self.shifts.iter().find(|s| s.id == *shift_id)
    }

    fn validate_id(&self, id: &ScheduleId) -> Result<(), ScheduleError> {
        match self.id == *id {
            true => Ok(()),
            false => Err(ScheduleError::MismatchedId),
        }
    }

    fn validate_shift_added(&self, shift: &Shift) -> Result<(), ScheduleError> {
        self.validate_duplicate_shift(&shift.id)?;
        self.validate_overlapping_shift(&shift.time)
    }

    fn validate_shift_time_changed(
        &self,
        shift_id: &ShiftId,
        time: &Range<DateTime<Utc>>,
    ) -> Result<(), ScheduleError> {
        self.validate_shift_not_found(shift_id)?;
        self.validate_overlapping_shift(time)?;
        match self.shift(shift_id) {
            Some(shift) => Ok(shift.validate_time(time)?),
            None => Err(ScheduleError::ShiftNotFound),
        }
    }

    fn validate_shift_status_changed(
        &self,
        shift_id: &ShiftId,
        status: &ShiftStatus,
    ) -> Result<(), ScheduleError> {
        self.validate_shift_not_found(shift_id)?;
        match self.shift(shift_id) {
            Some(shift) => Ok(shift.validate_status(status)?),
            None => Err(ScheduleError::ShiftNotFound),
        }
    }

    fn validate_shifts_deleted<'a, T: IntoIterator<Item = &'a ShiftId>>(
        &self,
        shift_ids: T,
    ) -> Result<(), ScheduleError> {
        let ids = shift_ids.into_iter().collect::<Vec<_>>();
        match self.shifts.iter().find(|s| ids.contains(&&s.id)) {
            Some(_) => Ok(()),
            None => Err(ScheduleError::ShiftNotFound),
        }
    }

    fn validate_duplicate_shift(&self, shift_id: &ShiftId) -> Result<(), ScheduleError> {
        match self.shifts.iter().find(|s| s.id == *shift_id) {
            Some(_) => Err(ScheduleError::DuplicateShift),
            None => Ok(()),
        }
    }

    fn validate_overlapping_shift(&self, time: &Range<DateTime<Utc>>) -> Result<(), ScheduleError> {
        match IntervalTree::from_iter(self.shifts.iter().map(|s| (&s.time, s)))
            .find(time)
            .next()
        {
            Some(_) => Err(ScheduleError::OverlappingShift),
            None => Ok(()),
        }
    }

    fn validate_shift_not_found(&self, shift_id: &ShiftId) -> Result<(), ScheduleError> {
        match self.shifts.iter().find(|s| s.id == *shift_id) {
            Some(_) => Ok(()),
            None => Err(ScheduleError::ShiftNotFound),
        }
    }
}

impl Entity for Schedule {
    type Id = ScheduleId;

    const ENTITY_NAME: &'static str = "schedule";

    fn id(&self) -> Self::Id {
        self.id
    }
}

impl Aggregation for Schedule {
    type Event = ScheduleEvent;
    type Error = ScheduleError;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error> {
        match event {
            ScheduleEvent::ScheduleCreated { .. } => Ok(()),
            ScheduleEvent::ScheduleDeleted { id } => self.validate_id(id),
            ScheduleEvent::ShiftAdded { id, shift } => {
                self.validate_id(id)?;
                self.validate_shift_added(shift)
            }
            ScheduleEvent::ShiftTimeChanged { shift_id, time } => {
                self.validate_shift_time_changed(shift_id, time)
            }
            ScheduleEvent::ShiftStatusChanged { shift_id, status } => {
                self.validate_shift_status_changed(shift_id, status)
            }
            ScheduleEvent::ShiftsDeleted { shift_ids } => self.validate_shifts_deleted(shift_ids),
        }
    }

    fn apply(&mut self, event: Self::Event) {
        match event {
            ScheduleEvent::ScheduleCreated { id, prostitute_id } => {
                if self.id != id {
                    *self = Self::create(id, prostitute_id);
                }
            }
            ScheduleEvent::ScheduleDeleted { .. } => {}
            ScheduleEvent::ShiftAdded { id, shift } => {
                if self.id == id {
                    if let Err(_e) = self.add_shift(shift) {}
                }
            }
            ScheduleEvent::ShiftTimeChanged { shift_id, time } => {
                if let Err(_e) = self.change_shift_time(shift_id, time) {}
            }
            ScheduleEvent::ShiftStatusChanged { shift_id, status } => {
                if let Err(_e) = self.change_shift_status(shift_id, status) {}
            }
            ScheduleEvent::ShiftsDeleted { shift_ids } => {
                if let Err(_e) = self.delete_shifts(shift_ids) {}
            }
        }
    }

    fn events(&self) -> &EventQueue<Self::Event> {
        &self.events
    }

    fn events_mut(&mut self) -> &mut EventQueue<Self::Event> {
        &mut self.events
    }
}

impl PartialEq for Schedule {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.prostitute_id == other.prostitute_id
            && self.shifts == other.shifts
    }
}

impl Eq for Schedule {}

#[derive(Error, Display, Debug, From)]
pub enum ScheduleError {
    #[display(fmt = "Mismatched id")]
    MismatchedId,
    #[display(fmt = "The shift does not exist in the schedule")]
    ShiftNotFound,
    #[display(fmt = "The schedule for this shift already exists")]
    DuplicateShift,
    #[display(fmt = "Shift overlaps with an existing shift")]
    OverlappingShift,
    #[display(fmt = "Shift error")]
    ShiftError(ShiftError),
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Display, From, Deref, Default, Hash,
)]
pub struct ShiftId(u64);

impl Id for ShiftId {
    type Inner = u64;
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub struct Shift {
    id: ShiftId,
    time: Range<DateTime<Utc>>,
    status: ShiftStatus,
}

impl Shift {
    pub fn create(
        id: ShiftId,
        time: Range<DateTime<Utc>>,
        status: ShiftStatus,
    ) -> Result<Self, ShiftError> {
        let entity = Shift { id, time, status };
        entity.validate(&entity)?;
        Ok(entity)
    }

    pub fn change_time(&mut self, time: Range<DateTime<Utc>>) -> Result<(), ShiftError> {
        self.validate_time(&time)?;
        self.time = time;
        Ok(())
    }

    pub fn change_status(&mut self, status: ShiftStatus) -> Result<(), ShiftError> {
        self.validate_status(&status)?;
        self.status = status;
        Ok(())
    }

    pub fn time(&self) -> Range<DateTime<Utc>> {
        self.time.clone()
    }

    pub fn status(&self) -> ShiftStatus {
        self.status
    }

    pub fn duration(&self) -> chrono::Duration {
        self.time.end - self.time.start
    }

    fn validate(&self, other: &Self) -> Result<(), ShiftError> {
        self.validate_time(&other.time)?;
        self.validate_status(&other.status)
    }

    fn validate_time(&self, time: &Range<DateTime<Utc>>) -> Result<(), ShiftError> {
        if time.start > time.end {
            Err(ShiftError::InvalidDuration)
        } else {
            Ok(())
        }
    }

    fn validate_status(&self, status: &ShiftStatus) -> Result<(), ShiftError> {
        match (&self.status, status) {
            (ShiftStatus::Editing, ShiftStatus::Reviewing)
            | (ShiftStatus::Editing, ShiftStatus::Confirmed)
            | (ShiftStatus::Reviewing, ShiftStatus::Editing)
            | (ShiftStatus::Reviewing, ShiftStatus::Confirmed)
            | (ShiftStatus::Reviewing, ShiftStatus::Canceled)
            | (ShiftStatus::Confirmed, ShiftStatus::Editing)
            | (ShiftStatus::Confirmed, ShiftStatus::Canceled)
            | (ShiftStatus::Canceled, ShiftStatus::Editing) => Ok(()),
            _ => Err(ShiftError::InvalidStatusTransition),
        }
    }
}

impl Entity for Shift {
    type Id = ShiftId;

    const ENTITY_NAME: &'static str = "shift";

    fn id(&self) -> ShiftId {
        self.id
    }
}

#[derive(Error, Display, Debug)]
pub enum ShiftError {
    #[display(fmt = "Invalid duration")]
    InvalidDuration,
    #[display(fmt = "Invalid status transition")]
    InvalidStatusTransition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ShiftStatus {
    /// 編集中
    Editing,
    /// 確認中
    Reviewing,
    /// 確定、
    Confirmed,
    /// キャンセル
    Canceled,
}

impl Default for ShiftStatus {
    fn default() -> Self {
        ShiftStatus::Editing
    }
}
