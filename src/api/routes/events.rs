use crate::api::auth::DatabaseAuth;
use crate::api::models::ApiResponse;
use crate::events::{EventSubscription, EventType, get_event_manager, Event};
use rocket::serde::json::Json;
use rocket_ws::{Channel, Message, WebSocket};
use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use tokio::sync::mpsc;

/// Connection response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionResponse {
    /// Connection status
    pub status: String,
    /// Connected database
    pub database: String,
    /// Table (optional)
    pub table: Option<String>,
}

/// WebSocket route for receiving real-time events
#[rocket::get("/events")]
pub fn events_ws(
    auth: DatabaseAuth,
    ws: WebSocket,
) -> Channel<'static> {
    println!("Receiving WebSocket connection");
    println!("Database: {}", auth.db_name);
    println!("Username: {}", auth.username);
    
    ws.channel(move |mut channel| {
        Box::pin(async move {
            let event_manager = get_event_manager();
            
            // Create a channel to receive events directly
            let (tx, mut rx) = mpsc::channel::<Event>(100);
            
            // Create a subscription for TableUpdate
            let update_subscription = EventSubscription {
                event_type: EventType::TableUpdate,
                database: Some(auth.db_name.clone()),
                table: None, // Receives events from all tables
            };
            println!("[DEBUG] Creating subscription for TableUpdate: {:?}", update_subscription);
            let mut update_receiver = event_manager.subscribe(update_subscription);
            
            // Create a subscription for TablePersist
            let persist_subscription = EventSubscription {
                event_type: EventType::TablePersist,
                database: Some(auth.db_name.clone()),
                table: None, // Receives events from all tables
            };
            println!("[DEBUG] Creating subscription for TablePersist: {:?}", persist_subscription);
            let mut persist_receiver = event_manager.subscribe(persist_subscription);
            
            // Spawn a task to process both receivers
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                println!("[DEBUG] Starting task to receive events");
                
                loop {
                    tokio::select! {
                        // Receive TableUpdate events
                        update_result = update_receiver.recv() => {
                            match update_result {
                                Ok(event) => {
                                    println!("[DEBUG] Received TableUpdate event: {:?}", event.event_type);
                                    if tx_clone.send(event).await.is_err() {
                                        println!("[DEBUG] Error sending TableUpdate event to main channel");
                                        break;
                                    }
                                },
                                Err(e) => {
                                    println!("[DEBUG] Error receiving TableUpdate event: {:?}", e);
                                    break;
                                }
                            }
                        },
                        
                        // Receive TablePersist events
                        persist_result = persist_receiver.recv() => {
                            match persist_result {
                                Ok(event) => {
                                    println!("[DEBUG] Received TablePersist event: {:?}", event.event_type);
                                    if tx_clone.send(event).await.is_err() {
                                        println!("[DEBUG] Error sending TablePersist event to main channel");
                                        break;
                                    }
                                },
                                Err(e) => {
                                    println!("[DEBUG] Error receiving TablePersist event: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                
                println!("[DEBUG] Finishing event receiving task");
            });
            
            // Send connection confirmation
            let response = ConnectionResponse {
                status: "connected".to_string(),
                database: auth.db_name.clone(),
                table: None,
            };
            
            if let Err(e) = channel.send(Message::Text(serde_json::to_string(&response).unwrap())).await {
                eprintln!("Error sending connection confirmation: {}", e);
                return Ok(());
            }
            
            // Main loop to receive events
            loop {
                tokio::select! {
                    // Receive events from mpsc channel
                    Some(event) = rx.recv() => {
                        println!("[DEBUG] Main loop: Sending event to client: {:?}", event.event_type);
                        // Send the event to the client
                        if let Err(e) = channel.send(Message::Text(serde_json::to_string(&event).unwrap())).await {
                            eprintln!("Error sending event: {}", e);
                            break;
                        }
                    },
                    
                    // Receive messages from client (only to detect disconnection)
                    message = channel.next() => {
                        match message {
                            Some(Ok(Message::Close(_))) | None => {
                                // Client disconnected
                                println!("Client disconnected");
                                break;
                            },
                            Some(Err(e)) => {
                                eprintln!("Error receiving message from client: {}", e);
                                break;
                            },
                            _ => {} // Ignore other messages
                        }
                    }
                }
            }
            
            Ok(())
        })
    })
}

/// Route to list available event types
#[rocket::get("/events/types")]
pub fn event_types() -> Json<ApiResponse<Vec<EventType>>> {
    let event_types = vec![
        EventType::TableUpdate,
        EventType::TablePersist,
    ];
    
    Json(ApiResponse {
        success: true,
        message: None,
        data: Some(event_types),
    })
} 