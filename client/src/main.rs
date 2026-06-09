use eframe::egui;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::protocol::Message;
use shared::ChatMessage;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Local;

struct ChatApp {
    tx: mpsc::UnboundedSender<String>,
    username: String,
    current_message: String,
    active_room: String,
    all_messages: Arc<Mutex<Vec<ChatMessage>>>,
    connection_status: Arc<Mutex<String>>,
}

impl ChatApp {
    fn new(tx: mpsc::UnboundedSender<String>, all_messages: Arc<Mutex<Vec<ChatMessage>>>, connection_status: Arc<Mutex<String>>) -> Self {
        Self {
            tx,
            username: format!("User_{}", rand::random::<u16>() % 1000),
            current_message: String::new(),
            active_room: "#genel".to_string(),
            all_messages,
            connection_status,
        }
    }
}

impl eframe::App for ChatApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🏁 Rust Real-Time WebChat - Final Sürümü");
            
            ui.horizontal(|ui| {
                ui.label("Profilin:");
                ui.text_edit_singleline(&mut self.username);
                ui.separator();
                let status = self.connection_status.lock().unwrap();
                ui.label(format!("Ağ Durumu: {}", *status));
            });

            ui.separator();

            ui.columns(2, |columns| {
                columns[0].vertical(|ui| {
                    ui.label("🗂️ Sohbet Odaları");
                    ui.separator();
                    
                    if ui.selectable_label(self.active_room == "#genel", "🌐 #genel").clicked() {
                        self.active_room = "#genel".to_string();
                    }
                    if ui.selectable_label(self.active_room == "#yazılım", "💻 #yazılım").clicked() {
                        self.active_room = "#yazılım".to_string();
                    }
                    if ui.selectable_label(self.active_room == "#proje-konseyi", "👑 #proje-konseyi").clicked() {
                        self.active_room = "#proje-konseyi".to_string();
                    }
                });

                columns[1].vertical(|ui| {
                    ui.label(format!("💬 Canlı Akış: {}", self.active_room));
                    ui.separator();

                    egui::ScrollArea::vertical().id_source("chat_scroll").show(ui, |ui| {
                        let msgs = self.all_messages.lock().unwrap();
                        for msg in msgs.iter().filter(|m| m.room == self.active_room) {
                           
                            ui.label(format!(" [{}] {}: {}", msg.timestamp, msg.sender, msg.content));
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let res = ui.text_edit_singleline(&mut self.current_message);
                        if ui.button("Gönder").clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            if !self.current_message.is_empty() {
                                // CİLA: Mesajın gönderildiği milisaniyedeki yerel saati alıyoruz (Örn: "20:05")
                                let now = Local::now().format("%H:%M").to_string();

                                let chat_msg = ChatMessage {
                                    sender: self.username.clone(),
                                    content: self.current_message.clone(),
                                    room: self.active_room.clone(),
                                    timestamp: now, 
                                };
                                if let Ok(serialized) = serde_json::to_string(&chat_msg) {
                                    let _ = self.tx.send(serialized);
                                }
                                self.current_message.clear();
                            }
                        }
                    });
                });
            });
        });
        ctx.request_repaint();
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    shared::init_logger();

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let all_messages = Arc::new(Mutex::new(Vec::new()));
    let connection_status = Arc::new(Mutex::new("Bağlanıyor...".to_string()));

    let all_messages_clone = Arc::clone(&all_messages);
    let status_clone = Arc::clone(&connection_status);

    let outbound_buffer = Arc::new(Mutex::new(Vec::new()));
    let outbound_buffer_clone = Arc::clone(&outbound_buffer);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            outbound_buffer_clone.lock().unwrap().push(msg);
        }
    });

    tokio::spawn(async move {
        let url = "ws://127.0.0.1:8080";
        loop {
            if let Ok((ws_stream, _)) = connect_async(url).await {
                *status_clone.lock().unwrap() = "🟢 Bağlı".to_string();
                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                let outbound_buffer_inner = Arc::clone(&outbound_buffer);
                let mut send_task = tokio::spawn(async move {
                    loop {
                        let mut msg_to_send = None;
                        {
                            let mut buffer = outbound_buffer_inner.lock().unwrap();
                            if !buffer.is_empty() {
                                msg_to_send = Some(buffer.remove(0));
                            }
                        }

                        if let Some(msg) = msg_to_send {
                            if ws_sender.send(Message::Text(msg)).await.is_err() {
                                break;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                });

                let all_msgs_inner = Arc::clone(&all_messages_clone);
                let status_inner = Arc::clone(&status_clone);
                let mut recv_task = tokio::spawn(async move {
                    while let Some(Ok(msg)) = ws_receiver.next().await {
                        if let Ok(text) = msg.into_text() {
                            if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                                all_msgs_inner.lock().unwrap().push(chat_msg);
                            }
                        }
                    }
                    *status_inner.lock().unwrap() = "🔴 Bağlantı Koptu".to_string();
                });

                tokio::select! {
                    _ = &mut send_task => {}
                    _ = &mut recv_task => {}
                }
            }
            *status_clone.lock().unwrap() = "🔄 Yeniden Bağlanıyor...".to_string();
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Chat Client Pro",
        options,
        Box::new(|_cc| Box::new(ChatApp::new(tx, all_messages, connection_status))),
    )
}