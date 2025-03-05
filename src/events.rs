use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Tipos de events that can be emitted by the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Event emitted when a record is updated
    TableUpdate,
    /// Event emitted when a new record is persisted
    TablePersist,
    /// Event emitted when a query is performed
    TableQuery,
}

/// Structure representing an event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Type of event
    pub event_type: EventType,
    /// Database name
    pub database: String,
    /// Table name
    pub table: String,
    /// Associated data with the event (optional)
    pub data: Option<serde_json::Value>,
    /// Event timestamp
    pub timestamp: u64,
}

impl Event {
    /// Creates a new event
    pub fn new(
        event_type: EventType,
        database: &str,
        table: &str,
        data: Option<serde_json::Value>,
    ) -> Self {
        Self {
            event_type,
            database: database.to_string(),
            table: table.to_string(),
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Event manager for the system
#[derive(Debug, Clone)]
pub struct EventManager {
    /// Broadcast channels for each type of event
    channels: Arc<Mutex<HashMap<EventSubscription, broadcast::Sender<Event>>>>,
}

/// Subscription key for events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventSubscription {
    /// Type of event
    pub event_type: EventType,
    /// Database name (optional)
    pub database: Option<String>,
    /// Table name (optional)
    pub table: Option<String>,
}

impl EventManager {
    /// Creates a new event manager
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Emits an event to all subscribers
    pub fn emit(&self, event: Event) {
        let channels = self.channels.lock().unwrap();

        println!(
            "[DEBUG] Emitting event: {:?} for database: {} table: {}",
            event.event_type, event.database, event.table
        );

        // Check if there is a specific subscription for this event
        let specific_subscription = EventSubscription {
            event_type: event.event_type.clone(),
            database: Some(event.database.clone()),
            table: Some(event.table.clone()),
        };

        // Check if there is a database-level subscription
        let db_subscription = EventSubscription {
            event_type: event.event_type.clone(),
            database: Some(event.database.clone()),
            table: None,
        };

        // Check if there is a global subscription
        let global_subscription = EventSubscription {
            event_type: event.event_type.clone(),
            database: None,
            table: None,
        };

        // Send the event only to existing subscriptions
        // First check if there is a specific subscription
        if let Some(sender) = channels.get(&specific_subscription) {
            println!(
                "[DEBUG] Sending event to specific subscription: {:?}",
                specific_subscription
            );
            let _ = sender.send(event.clone());
        }
        // If there is no specific subscription, check if there is a database-level subscription
        else if let Some(sender) = channels.get(&db_subscription) {
            println!(
                "[DEBUG] Sending event to database subscription: {:?}",
                db_subscription
            );
            let _ = sender.send(event.clone());
        }
        // If there is no database-level subscription, check if there is a global subscription
        else if let Some(sender) = channels.get(&global_subscription) {
            println!(
                "[DEBUG] Sending event to global subscription: {:?}",
                global_subscription
            );
            let _ = sender.send(event.clone());
        }
    }

    /// Subscribes to receive events of a specific type
    pub fn subscribe(&self, subscription: EventSubscription) -> broadcast::Receiver<Event> {
        let mut channels = self.channels.lock().unwrap();

        // Cria ou obtém o canal para esta inscrição
        let sender = channels
            .entry(subscription)
            .or_insert_with(|| broadcast::channel(100).0);

        // Retorna um receptor para o canal
        sender.subscribe()
    }

    /// Cancels the subscription for events
    pub fn unsubscribe(&self, subscription: &EventSubscription) {
        let mut channels = self.channels.lock().unwrap();
        channels.remove(subscription);
    }
}

// Singleton for the event manager
lazy_static::lazy_static! {
    static ref EVENT_MANAGER: EventManager = EventManager::new();
}

/// Gets the global instance of the event manager
pub fn get_event_manager() -> EventManager {
    EVENT_MANAGER.clone()
}

/// Emits an event to all subscribers
pub fn emit_event(event: Event) {
    get_event_manager().emit(event);
}

/// Emits a table update event
pub fn emit_table_update(database: &str, table: &str, data: Option<serde_json::Value>) {
    emit_event(Event::new(EventType::TableUpdate, database, table, data));
}

/// Emits a table persist event
pub fn emit_table_persist(database: &str, table: &str, data: Option<serde_json::Value>) {
    emit_event(Event::new(EventType::TablePersist, database, table, data));
}
