pub mod core;
pub mod customer;

use derive_more::{Display, From};
use once_cell::sync;
use serde::{Deserialize, Serialize};
use snowflake::SnowflakeIdGenerator;
use std::{
    collections::VecDeque,
    error,
    fmt::{Debug, Display},
    ops::Deref,
    str::FromStr,
    sync::Arc,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

pub trait Id:
    Copy
    + Eq
    + Deref<Target = Self::Inner>
    + From<Self::Inner>
    + Display
    + Debug
    + Serialize
    + for<'de> Deserialize<'de>
{
    type Inner: FromStr;
}

pub trait Event: Clone + Eq + Debug + Serialize + for<'de> Deserialize<'de> {
    type Id;
}

pub trait Entity: Eq + Debug + Default + Clone + Serialize + for<'de> Deserialize<'de> {
    type Id: Id;

    const ENTITY_NAME: &'static str;

    fn id(&self) -> Self::Id;
}

pub trait Aggregation: Entity +
    IntoIterator<Item = Self::Event>
{
    type Event: Event<Id = Self::Id>;
    type Error: error::Error;

    fn validate(&self, event: &Self::Event) -> Result<(), Self::Error>;
    fn apply(&mut self, event: Self::Event);
    fn events(&self) -> &EventQueue<Self::Event>;
    fn events_mut(&mut self) -> &mut EventQueue<Self::Event>;
    fn pop(&mut self) -> Option<Self::Event> {
        self.events_mut().pop()
    }
    fn pop_all(&mut self) -> Vec<Self::Event> {
        let mut events = Vec::new();
        while let Some(e) = self.pop() {
            events.push(e);
        }
        events
    }
    fn clear(&mut self) {
        self.events_mut().clear()
    }
    fn peek(&self) -> Option<&Self::Event> {
        self.events().peek()
    }
    fn iter(&self) -> EventQueueIter<'_, Self::Event> {
        self.events().iter()
    }
    fn iter_mut(&mut self) -> EventQueueIterMut<'_, Self::Event> {
        self.events_mut().iter_mut()
    }
}

#[derive(Display, Debug)]
pub enum DataAccessError {
    #[display(fmt = "Database connection error: {}", "_0.to_string()")]
    ConnectionError(Box<dyn error::Error>),
    #[display(fmt = "Database query error: {}", "_0.to_string()")]
    QueryError(Box<dyn error::Error>),
    #[display(fmt = "Data read error: {}", "_0.to_string()")]
    ReadError(Box<dyn error::Error>),
    #[display(fmt = "Data write error: {}", "_0.to_string()")]
    WriteError(Box<dyn error::Error>),
    #[display(fmt = "Client side error: {}", "_0.to_string()")]
    ClientSideError(Box<dyn error::Error>),
}

impl error::Error for DataAccessError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DataAccessError::ConnectionError(e) => Some(e.as_ref()),
            DataAccessError::QueryError(e) => Some(e.as_ref()),
            DataAccessError::ReadError(e) => Some(e.as_ref()),
            DataAccessError::WriteError(e) => Some(e.as_ref()),
            DataAccessError::ClientSideError(e) => Some(e.as_ref()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventQueue<T> {
    queue: VecDeque<T>,
}

impl<T> EventQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
    pub fn peek(&self) -> Option<&T> {
        self.queue.front()
    }
    pub fn push(&mut self, value: T) {
        self.queue.push_back(value)
    }
    pub fn pop(&mut self) -> Option<T> {
        self.queue.pop_front()
    }
    pub fn clear(&mut self) {
        self.queue.clear()
    }
    pub fn iter(&self) -> EventQueueIter<'_, T> {
        self.queue.iter()
    }
    pub fn iter_mut(&mut self) -> EventQueueIterMut<'_, T> {
        self.queue.iter_mut()
    }
}

impl<T> IntoIterator for EventQueue<T> {
    type Item = T;
    type IntoIter = EventQueueIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.queue.into_iter()
    }
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub type EventQueueIntoIter<T> = std::collections::vec_deque::IntoIter<T>;
pub type EventQueueIter<'a, T> = std::collections::vec_deque::Iter<'a, T>;
pub type EventQueueIterMut<'a, T> = std::collections::vec_deque::IterMut<'a, T>;

#[derive(From)]
pub struct IdGenerator(SnowflakeIdGenerator);

impl IdGenerator {
    pub fn generate(&mut self) -> u64 {
        self.0.generate() as u64
    }

    pub fn new(gen: SnowflakeIdGenerator) -> Self {
        Self(gen)
    }
}

pub static ID_GENERATOR: sync::Lazy<IdGeneratorTask> =
    sync::Lazy::new(|| IdGeneratorTask::spawn(SnowflakeIdGenerator::new(1, 1).into()));

#[derive(Clone)]
pub struct IdGeneratorTask {
    _handle: Arc<JoinHandle<()>>,
    sender: mpsc::Sender<oneshot::Sender<u64>>,
}

impl IdGeneratorTask {
    pub fn spawn(mut gen: IdGenerator) -> Self {
        let (tx_async, mut rx_async) = mpsc::channel::<oneshot::Sender<u64>>(100);
        let handle = tokio::spawn(async move {
            while let Some(tx) = rx_async.recv().await {
                tx.send(gen.generate()).unwrap();
            }
        });
        Self {
            _handle: Arc::new(handle),
            sender: tx_async,
        }
    }

    pub async fn generate<T>(&self) -> T
    where
        T: From<u64>,
    {
        let (tx, rx) = oneshot::channel::<u64>();
        self.sender.send(tx).await.unwrap();
        T::from(rx.await.unwrap())
    }
}
