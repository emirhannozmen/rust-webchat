use eframe::egui;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::protocol::Message;

struct ChatApp {
    tx: mpsc::UnboundedSender<String>,
    connection_status: String,
}

impl ChatApp {
    fn new(tx: mpsc::UnboundedSender<String>) -> Self {
        Self {
            tx,
            connection_status: "Sunucuya bağlanılıyor...".to_string(),
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Real-Time WebChat - Sprint 1");
            ui.label(format!("Durum: {}", self.connection_status));

            ui.separator();

            if ui.button("Test Mesajı Gönder").clicked() {
                let _ = self.tx.send("Merhaba Sunucu! Ben Egui İstemcisi.".to_string());
                self.connection_status = "Mesaj gönderildi!".to_string();
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    shared::init_logger();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        let url = "ws://127.0.0.1:8080";
        if let Ok((mut ws_stream, _)) = connect_async(url).await {
            println!("🌐 Arka Plan: WebSocket sunucusuna bağlanıldı!");
            while let Some(msg_content) = rx.recv().await {
                let _ = ws_stream.send(Message::Text(msg_content)).await;
            }
        } else {
            eprintln!("❌ Arka Plan: Sunucuya bağlanılamadı.");
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Chat Client",
        options,
        Box::new(|_cc| Box::new(ChatApp::new(tx))),
    )
}