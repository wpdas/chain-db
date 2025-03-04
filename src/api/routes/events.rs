use crate::api::auth::DatabaseAuth;
use crate::api::models::ApiResponse;
use crate::events::{EventSubscription, EventType, get_event_manager};
use rocket::serde::json::Json;
use rocket_ws::{Channel, Message, WebSocket};
use serde::{Deserialize, Serialize};
use futures::stream::StreamExt;
use futures::sink::SinkExt;

/// Estrutura para a requisição de inscrição em eventos
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeRequest {
    /// Tipo do evento
    pub event_type: EventType,
    /// Nome da tabela (opcional)
    pub table: Option<String>,
}

/// Estrutura para a resposta de inscrição em eventos
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeResponse {
    /// ID da inscrição
    pub subscription_id: String,
}

/// Rota WebSocket para receber eventos em tempo real
#[rocket::get("/events")]
pub fn events_ws(
    auth: DatabaseAuth,
    ws: WebSocket,
) -> Channel<'static> {
    println!("Recebendo conexão WebSocket");
    println!("Database: {}", auth.db_name);
    println!("Username: {}", auth.username);
    println!("Password: {}", auth.password);
    
    ws.channel(move |mut channel| {
        Box::pin(async move {
            let event_manager = get_event_manager();
            
            // Aguarda a primeira mensagem do cliente, que deve ser a requisição de inscrição
            if let Some(Ok(Message::Text(message))) = channel.next().await {
                // Tenta deserializar a mensagem como uma requisição de inscrição
                match serde_json::from_str::<SubscribeRequest>(&message) {
                    Ok(request) => {
                        // Cria a inscrição
                        let subscription = EventSubscription {
                            event_type: request.event_type,
                            database: Some(auth.db_name.clone()),
                            table: request.table,
                        };
                        
                        // Inscreve-se para receber eventos
                        let mut receiver = event_manager.subscribe(subscription.clone());
                        
                        // Envia confirmação de inscrição
                        let subscription_id = format!("{:?}", subscription);
                        let response = SubscribeResponse {
                            subscription_id: subscription_id.clone(),
                        };
                        
                        if let Err(e) = channel.send(Message::Text(serde_json::to_string(&response).unwrap())).await {
                            eprintln!("Erro ao enviar confirmação de inscrição: {}", e);
                            return Ok(());
                        }
                        
                        // Loop principal para receber eventos e mensagens do cliente
                        loop {
                            tokio::select! {
                                // Recebe eventos do gerenciador de eventos
                                event = receiver.recv() => {
                                    match event {
                                        Ok(event) => {
                                            // Envia o evento para o cliente
                                            if let Err(e) = channel.send(Message::Text(serde_json::to_string(&event).unwrap())).await {
                                                eprintln!("Erro ao enviar evento: {}", e);
                                                break;
                                            }
                                        },
                                        Err(e) => {
                                            eprintln!("Erro ao receber evento: {}", e);
                                            break;
                                        }
                                    }
                                },
                                
                                // Recebe mensagens do cliente
                                message = channel.next() => {
                                    match message {
                                        Some(Ok(Message::Text(message))) => {
                                            // Tenta deserializar a mensagem como uma requisição de inscrição
                                            match serde_json::from_str::<SubscribeRequest>(&message) {
                                                Ok(request) => {
                                                    // Cria a inscrição
                                                    let new_subscription = EventSubscription {
                                                        event_type: request.event_type,
                                                        database: Some(auth.db_name.clone()),
                                                        table: request.table,
                                                    };
                                                    
                                                    // Inscreve-se para receber eventos
                                                    receiver = event_manager.subscribe(new_subscription.clone());
                                                    
                                                    // Envia confirmação de inscrição
                                                    let subscription_id = format!("{:?}", new_subscription);
                                                    let response = SubscribeResponse {
                                                        subscription_id: subscription_id.clone(),
                                                    };
                                                    
                                                    if let Err(e) = channel.send(Message::Text(serde_json::to_string(&response).unwrap())).await {
                                                        eprintln!("Erro ao enviar confirmação de inscrição: {}", e);
                                                        break;
                                                    }
                                                },
                                                Err(e) => {
                                                    eprintln!("Erro ao deserializar requisição de inscrição: {}", e);
                                                    if let Err(e) = channel.send(Message::Text(format!("Erro: {}", e))).await {
                                                        eprintln!("Erro ao enviar mensagem de erro: {}", e);
                                                    }
                                                    break;
                                                }
                                            }
                                        },
                                        Some(Ok(Message::Close(_))) | None => {
                                            // Cliente desconectou
                                            break;
                                        },
                                        Some(Err(e)) => {
                                            eprintln!("Erro ao receber mensagem do cliente: {}", e);
                                            break;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Erro ao deserializar requisição de inscrição: {}", e);
                        let _ = channel.send(Message::Text(format!("Erro: {}", e))).await;
                    }
                }
            }
            
            Ok(())
        })
    })
}

/// Rota para listar os tipos de eventos disponíveis
#[rocket::get("/events/types")]
pub fn event_types() -> Json<ApiResponse<Vec<EventType>>> {
    let event_types = vec![
        EventType::TableUpdate,
        EventType::TablePersist,
        EventType::TableQuery,
    ];
    
    Json(ApiResponse {
        success: true,
        message: None,
        data: Some(event_types),
    })
} 