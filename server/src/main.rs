use tokio::net::TcpListener;
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};
use shared::{init_logger, ChatMessage};
use log::{info, error};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 Sprint 3: Sohbet Odası Destekli Sunucu {} adresinde başladı...", addr);

    let (tx, _) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        info!("🔌 Yeni bağlantı: {}", peer_addr);

        let tx = Arc::clone(&tx);
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            if let Ok(ws_stream) = tokio_tungstenite::accept_async(stream).await {
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                // Görev 1: Sunucudaki ana yayını dinle
                let mut send_task = tokio::spawn(async move {
                    while let Ok(msg_text) = rx.recv().await {
                        // Burada istemciye mesajı doğrudan üflüyoruz, filtrelemeyi istemci kendi odasına göre yapacak
                        if ws_sender.send(tokio_tungstenite::tungstenite::protocol::Message::Text(msg_text)).await.is_err() {
                            break;
                        }
                    }
                });

                // Görev 2: İstemciden gelen mesajı al ve yayın kanalına fırlat
                let tx_clone = Arc::clone(&tx);
                let mut recv_task = tokio::spawn(async move {
                    while let Some(Ok(msg)) = ws_receiver.next().await {
                        if let Ok(text) = msg.into_text() {
                            let _ = tx_clone.send(text);
                        }
                    }
                });

                tokio::select! {
                    _ = &mut send_task => {}
                    _ = &mut recv_task => {}
                }
                info!("❌ {} ayrıldı.", peer_addr);
            }
        });
    }
}