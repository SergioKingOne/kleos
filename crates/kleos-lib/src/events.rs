use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::marker::PhantomData;

/// A unique identifier for an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A generic payload wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload<T> {
    pub data: T,
}

impl<T> Payload<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

// --- State Types ---

/// State representing an event that has been created but not yet published.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Created;

/// State representing an event that has been consumed from a stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consumed;

/// The core event moving through the system.
/// The state `S` ensures correct lifecycle usage at compile time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event<T, S = Created> {
    pub id: EventId,
    pub payload: Payload<T>,
    pub created_at: DateTime<Utc>,
    pub metadata: std::collections::HashMap<String, String>,
    #[serde(skip)]
    pub _state: PhantomData<S>,
}

impl<T> Event<T, Created> {
    pub fn new(payload: Payload<T>) -> Self {
        Self {
            id: EventId::new(),
            payload,
            created_at: Utc::now(),
            metadata: std::collections::HashMap::new(),
            _state: PhantomData,
        }
    }
}

impl<T, S> Event<T, S> {
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Transitions the event to a new state.
    /// This is an internal helper to facilitate state transitions in the ports.
    pub fn transition<NewS>(self) -> Event<T, NewS> {
        Event {
            id: self.id,
            payload: self.payload,
            created_at: self.created_at,
            metadata: self.metadata,
            _state: PhantomData,
        }
    }
}
