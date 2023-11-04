use async_tungstenite::tokio::{connect_async, TokioAdapter};
use async_tungstenite::{WebSocketStream, stream::Stream as WebSocketStreamGeneric,tungstenite::protocol::WebSocket, tungstenite::{Message,Error as WsError}};
use futures_util::{SinkExt, StreamExt};
use tokio::{time::{Duration, self}, sync::Mutex};
use std::sync::Arc;

type GenericWebSocketStream = WebSocketStreamGeneric<TokioAdapter<tokio::net::TcpStream>, TokioAdapter<tokio_native_tls::TlsStream<tokio::net::TcpStream>>>;

pub struct WebSocketClient {
    pub ws_stream: Mutex<Option<WebSocketStream<GenericWebSocketStream>>>,
    pub url: Mutex<String>,
}

impl WebSocketClient {
    // Method to create a new WebSocket connection if necessary and store the URL
    pub async fn create_new(url: &str, ws_client: Arc<WebSocketClient>) -> Result<Arc<WebSocketClient>, WsError> {
        {
            // Lock the ws_stream to check the current WebSocket connection
            let ws_stream_guard = ws_client.ws_stream.lock().await;
            if ws_stream_guard.is_none() {
                // Drop the guard before establishing a new connection
                drop(ws_stream_guard);

                // There is no active WebSocket connection, so create one
                let (ws_stream, _) = connect_async(url).await?;
                let mut ws_stream_guard = ws_client.ws_stream.lock().await;
                *ws_stream_guard = Some(ws_stream);

                // Store the URL
                let mut url_guard = ws_client.url.lock().await;
                *url_guard = url.to_owned();
            }
            // If there's already a WebSocket, we do nothing
        }

        // Whether we created a new connection or not, we return the client
        Ok(ws_client.clone())
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