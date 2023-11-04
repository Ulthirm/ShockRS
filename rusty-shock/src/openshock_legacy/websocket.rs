use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::{WebSocketStream, stream::Stream as WebSocketStreamGeneric,tungstenite::protocol::WebSocket, tungstenite::{Message,Error as WsError}};
use futures_util::{SinkExt, StreamExt};
use tokio::{time::{Duration, self}, sync::{Mutex,OnceCell}};
use std::sync::Arc;
use once_cell::sync::Lazy;

type GenericWebSocketStream = WebSocketStreamGeneric<TokioAdapter<tokio::net::TcpStream>, TokioAdapter<tokio_native_tls::TlsStream<tokio::net::TcpStream>>>;

pub struct WebSocketClient {
    pub ws_stream: Mutex<Option<WebSocketStream<GenericWebSocketStream>>>,
    pub url: Mutex<String>,
}

// Lazy here is used to defer the creation of the client until it's actually needed.
static WEBSOCKET_CLIENT: Lazy<Mutex<Option<Arc<WebSocketClient>>>> = Lazy::new(|| Mutex::new(None));

impl WebSocketClient {
    // This should only take a URL and create a new WebSocketClient
    pub async fn create_new(url: &str) -> Result<WebSocketClient, WsError> {
        let (ws_stream, _) = connect_async(url).await?;
        
        Ok(WebSocketClient {
            ws_stream: Mutex::new(Some(ws_stream)),
            url: Mutex::new(url.to_owned()),
        })
    }

    // This should be an associated function, not an instance method.
    // TODO: ERROR HANDLING
    pub async fn get_or_init_websocket_client(url: &str) -> Result<Arc<WebSocketClient>, WsError> {
        let mut ws_client_guard = WEBSOCKET_CLIENT.lock().await;

        match &*ws_client_guard {
            Some(client) => Ok(client.clone()),
            None => {
                let client = WebSocketClient::create_new(url).await?;
                let client_arc = Arc::new(client);
                *ws_client_guard = Some(client_arc.clone());
                Ok(client_arc)
            }
        }
    }

    // Method to reconnect using the stored URL
    async fn reconnect(&self) -> Result<(), WsError> {
        let url = {
            // Access the URL within its Mutex
            let url_guard = self.url.lock().await;
            url_guard.clone()
        };
        
        let (ws_stream, _) = connect_async(&url).await?;

        let mut ws_stream_guard = self.ws_stream.lock().await;
        *ws_stream_guard = Some(ws_stream);

        Ok(())
    }

    pub async fn send(&self, message: String) -> Result<(), WsError> {
        let mut ws_stream = self.ws_stream.lock().await;
    
        // Try sending the message
        if let Some(stream) = ws_stream.as_mut() {
            if let Err(send_error) = stream.send(Message::Text(message.clone())).await {
                // Log the send error
                log::error!("Send error: {}", send_error);
    
                // Drop the current ws_stream to free the lock before reconnecting
                drop(ws_stream);
    
                // Try to reconnect
                if let Err(reconnect_error) = self.reconnect().await {
                    // Log the reconnect error
                    log::error!("Reconnect error: {}", reconnect_error);
                    return Err(reconnect_error);
                }
    
                // Re-acquire the lock and try to send the original message again
                ws_stream = self.ws_stream.lock().await;
                if let Some(stream) = ws_stream.as_mut() {
                    stream.send(Message::Text(message)).await?;
                } else {
                    // Handle the case where the stream is still None after reconnecting
                    return Err(WsError::ConnectionClosed);
                }
            }
        } else {
            // Handle the case where there was no connection to begin with
            return Err(WsError::ConnectionClosed);
        }
    
        Ok(())
    }
    

    pub async fn receive(&self) -> Option<Result<async_tungstenite::tungstenite::Message, async_tungstenite::tungstenite::Error>> {
        let mut ws_stream = self.ws_stream.lock().await;
        if let Some(stream) = ws_stream.as_mut() {
            stream.next().await
        } else {
            None
        }
    }

    pub async fn close(&self) -> Result<(), async_tungstenite::tungstenite::Error> {
        let mut ws_stream = self.ws_stream.lock().await;
        if let Some(stream) = ws_stream.as_mut() {
            stream.close(None).await
        } else {
            Err(async_tungstenite::tungstenite::Error::AlreadyClosed)
        }
    }
}