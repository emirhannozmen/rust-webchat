// server/src/main.rs
use tokio::net::TcpListener;
use futures_util::StreamExt;
use shared::init_logger;
use log::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    init_logger();
    
    let addr = "127.0.0.1:8080";
   
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 WebSocket Sunucusu {} adresinde dinlemeye başladı...", addr);

    
    loop {
       
        let (stream, peer_addr) = listener.accept().await?;
        info!("🔌 Yeni TCP bağlantısı algılandı: {}", peer_addr);

       
        tokio::spawn(async move {
           
            match tokio_tungstenite::accept_async(stream).await {
                Ok(mut ws_stream) => {
                    info!("✅ {} adresiyle WebSocket el sıkışması başarılı!", peer_addr);
                    
                   
                    if let Some(Ok(msg)) = ws_stream.next().await {
                        info!("📩 Gelen İlk Mesaj ({}): {}", peer_addr, msg);
                    }
                }
                Err(e) => error!("❌ WebSocket el sıkışma hatası ({}): {}", peer_addr, e),
            }
        });
    }
}