use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{ws::Message, Filter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use futures::{StreamExt, SinkExt};

// Client structure to hold connection information
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
struct Client {
    pub user_id: String,
    pub sender: Option<futures::channel::mpsc::UnboundedSender<Message>>,
}

// Chat message structure
#[derive(Serialize, Deserialize, Debug)]
struct ChatMessage {
    user_id: String,
    message: String,
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    let static_files = warp::path("static").and(warp::fs::dir("static"));
    let routes = ws_route.or(static_files);

    println!("Server started at http://localhost:8000/static/index.html");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn handle_connection(ws: warp::ws::WebSocket, clients: Clients) {
    let user_id = Uuid::new_v4().to_string();
    let (mut ws_sender, mut ws_rcv) = ws.split();
    
    let (tx, mut rx) = futures::channel::mpsc::unbounded();
    
    clients.write().await.insert(
        user_id.clone(),
        Client {
            user_id: user_id.clone(),
            sender: Some(tx),
        },
    );

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            ws_sender
                .send(message)
                .await
                .unwrap_or_else(|e| eprintln!("WebSocket send error: {}", e));
        }
    });
    
    while let Some(result) = ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        };
        
        if let Ok(message) = handle_message(&user_id, msg, &clients).await {
            broadcast_message(&clients, &message).await;
        }
    }

    clients.write().await.remove(&user_id);
}

async fn handle_message(user_id: &str, msg: Message, _clients: &Clients) -> Result<ChatMessage, Box<dyn std::error::Error + Send + Sync>> {
    let message = msg.to_str().unwrap_or_default();
    Ok(ChatMessage {
        user_id: user_id.to_string(),
        message: message.to_string(),
    })
}

async fn broadcast_message(clients: &Clients, message: &ChatMessage) {
    let message_json = serde_json::to_string(&message).unwrap();
    let message = Message::text(message_json);
    
    for client in clients.read().await.values() {
        if let Some(sender) = &client.sender {
            let _ = sender.unbounded_send(message.clone());
        }
    }
} 