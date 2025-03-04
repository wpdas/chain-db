use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Tipos de eventos que podem ser emitidos pelo sistema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    /// Evento emitido quando um registro é atualizado
    TableUpdate,
    /// Evento emitido quando um novo registro é persistido
    TablePersist,
    /// Evento emitido quando uma consulta é realizada
    TableQuery,
}

/// Estrutura que representa um evento no sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Tipo do evento
    pub event_type: EventType,
    /// Nome do banco de dados
    pub database: String,
    /// Nome da tabela
    pub table: String,
    /// Dados associados ao evento (opcional)
    pub data: Option<serde_json::Value>,
    /// Timestamp do evento
    pub timestamp: u64,
}

impl Event {
    /// Cria um novo evento
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

/// Gerenciador de eventos do sistema
#[derive(Debug, Clone)]
pub struct EventManager {
    /// Canais de broadcast para cada tipo de evento
    channels: Arc<Mutex<HashMap<EventSubscription, broadcast::Sender<Event>>>>,
}

/// Chave de inscrição para eventos
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EventSubscription {
    /// Tipo do evento
    pub event_type: EventType,
    /// Nome do banco de dados (opcional)
    pub database: Option<String>,
    /// Nome da tabela (opcional)
    pub table: Option<String>,
}

impl EventManager {
    /// Cria um novo gerenciador de eventos
    pub fn new() -> Self {
        Self {
            channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Emite um evento para todos os inscritos
    pub fn emit(&self, event: Event) {
        let channels = self.channels.lock().unwrap();

        // Lista de possíveis inscrições que correspondem a este evento
        let subscriptions = [
            // Inscrição específica (tipo + banco + tabela)
            EventSubscription {
                event_type: event.event_type.clone(),
                database: Some(event.database.clone()),
                table: Some(event.table.clone()),
            },
            // Inscrição por banco de dados (tipo + banco)
            EventSubscription {
                event_type: event.event_type.clone(),
                database: Some(event.database.clone()),
                table: None,
            },
            // Inscrição global por tipo de evento
            EventSubscription {
                event_type: event.event_type.clone(),
                database: None,
                table: None,
            },
        ];

        // Envia o evento para todos os canais correspondentes
        for subscription in &subscriptions {
            if let Some(sender) = channels.get(subscription) {
                // Ignora erros de envio (ocorrem quando não há receptores)
                let _ = sender.send(event.clone());
            }
        }
    }

    /// Inscreve-se para receber eventos de um tipo específico
    pub fn subscribe(&self, subscription: EventSubscription) -> broadcast::Receiver<Event> {
        let mut channels = self.channels.lock().unwrap();

        // Cria ou obtém o canal para esta inscrição
        let sender = channels
            .entry(subscription)
            .or_insert_with(|| broadcast::channel(100).0);

        // Retorna um receptor para o canal
        sender.subscribe()
    }

    /// Cancela a inscrição para eventos
    pub fn unsubscribe(&self, subscription: &EventSubscription) {
        let mut channels = self.channels.lock().unwrap();
        channels.remove(subscription);
    }
}

// Singleton para o gerenciador de eventos
lazy_static::lazy_static! {
    static ref EVENT_MANAGER: EventManager = EventManager::new();
}

/// Obtém a instância global do gerenciador de eventos
pub fn get_event_manager() -> EventManager {
    EVENT_MANAGER.clone()
}

/// Emite um evento para todos os inscritos
pub fn emit_event(event: Event) {
    get_event_manager().emit(event);
}

/// Emite um evento de atualização de tabela
pub fn emit_table_update(database: &str, table: &str, data: Option<serde_json::Value>) {
    emit_event(Event::new(EventType::TableUpdate, database, table, data));
}

/// Emite um evento de persistência de tabela
pub fn emit_table_persist(database: &str, table: &str, data: Option<serde_json::Value>) {
    emit_event(Event::new(EventType::TablePersist, database, table, data));
}

/// Emite um evento de consulta de tabela
pub fn emit_table_query(database: &str, table: &str, data: Option<serde_json::Value>) {
    emit_event(Event::new(EventType::TableQuery, database, table, data));
}
