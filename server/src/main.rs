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
    info!("🚀 Sprint 2: Çoklu Kullanıcı Destekli Sunucu {} adresinde başladı...", addr);

    let (tx, _) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        info!("🔌 Yeni bağlantı isteği: {}", peer_addr);

        let tx = Arc::clone(&tx);
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            match tokio_tungstenite::accept_async(stream).await {
                Ok(ws_stream) => {
                    info!("✅ {} ile WebSocket el sıkışması başarılı!", peer_addr);
                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                    // Görev 1: Sunucunun ana kanalına yeni mesaj düştüğünde, bunu bu istemciye WebSocket ile üfle
                    let mut send_task = tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            if ws_sender.send(tokio_tungstenite::tungstenite::protocol::Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                    });

                    // Görev 2: Bu istemciden yeni bir WebSocket mesajı geldiğinde, onu al ve dağıt
                    let tx_clone = Arc::clone(&tx);
                    let mut recv_task = tokio::spawn(async move {
                        while let Some(Ok(msg)) = ws_receiver.next().await {
                            if let Ok(text) = msg.into_text() {
                                let _ = tx_clone.send(text);
                            }
                        }
                    });

                    // İki görevden biri patlarsa (kullanıcı çıkarsa) bağlantıyı temizle
                    tokio::select! {
                        _ = &mut send_task => {}
                        _ = &mut recv_task => {}
                    }
                    info!("❌ {} sunucudan ayrıldı.", peer_addr);
                }
                Err(e) => error!("❌ El sıkışma hatası ({}): {}", peer_addr, e),
            }
        });
    }
}