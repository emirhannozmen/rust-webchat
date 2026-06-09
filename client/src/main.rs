use eframe::egui;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::protocol::Message;
use shared::ChatMessage;
use std::sync::{Arc, Mutex};

struct ChatApp {
    tx: mpsc::UnboundedSender<String>,
    username: String,
    current_message: String,
    logs: Arc<Mutex<Vec<String>>>,
}

impl ChatApp {
    fn new(tx: mpsc::UnboundedSender<String>, logs: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            tx,
            username: format!("Kullanıcı_{}", rand::random::<u8>()),
            current_message: String::new(),
            logs,
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🚀 Rust Real-Time WebChat - Sprint 2");
            
            ui.horizontal(|ui| {
                ui.label("Kullanıcı Adı:");
                ui.text_edit_singleline(&mut self.username);
            });

            ui.separator();

            ui.label("💬 Sohbet Geçmişi:");
            // DÜZELTME: autohide fonksiyonunu kaldırarak uyumsuzluğu giderdik
            egui::ScrollArea::vertical().show(ui, |ui| {
                let logs = self.logs.lock().unwrap();
                for log in logs.iter() {
                    ui.label(log);
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(&mut self.current_message);
                if ui.button("Gönder").clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if !self.current_message.is_empty() {
                        let chat_msg = ChatMessage {
                            sender: self.username.clone(),
                            content: self.current_message.clone(),
                        };
                        if let Ok(serialized) = serde_json::to_string(&chat_msg) {
                            let _ = self.tx.send(serialized);
                        }
                        self.current_message.clear();
                    }
                }
            });
        });
        
        ctx.request_repaint();
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    shared::init_logger();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = Arc::clone(&logs);

    tokio::spawn(async move {
        let url = "ws://127.0.0.1:8080";
        if let Ok((mut ws_stream, _)) = connect_async(url).await {
            let (mut ws_sender, mut ws_receiver) = ws_stream.split();

            tokio::spawn(async move {
                while let Some(msg_content) = rx.recv().await {
                    let _ = ws_sender.send(Message::Text(msg_content)).await;
                }
            });

            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Ok(text) = msg.into_text() {
                    if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                        let mut logs = logs_clone.lock().unwrap();
                        logs.push(format!("{}: {}", chat_msg.sender, chat_msg.content));
                    }
                }
            }
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Chat Client",
        options,
        Box::new(|_cc| Box::new(ChatApp::new(tx, logs))),
    )
}