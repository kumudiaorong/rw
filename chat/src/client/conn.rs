use super::action::Action;
use crate::Message;
use rocket::serde::json::Json;
use std::error::Error;
use tokio::{
    sync::mpsc,
    time::{timeout, Duration},
};
pub struct Conn {
    client: reqwest::Client,
}
impl Conn {
    pub async fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    pub async fn send(&self, dst: u32, msg: &Message) -> Result<(), reqwest::Error> {
        self.client
            .post(format!("http://localhost:8000/send?dst={}", dst))
            .json(msg)
            .send()
            .await?;
        Ok(())
    }
    pub async fn recv(&self, id: u32) -> Result<Vec<Message>, reqwest::Error> {
        let resp = self
            .client
            .get(format!("http://localhost:8000/recv?dst={}", id))
            .send()
            .await?;
        let msgs = resp.json::<Vec<Message>>().await?;
        Ok(msgs)
    }
}

pub async fn new() -> Result<u32, Box<dyn Error>> {
    let msg = reqwest::get("http://localhost:8000/")
        .await?
        .text()
        .await?
        .parse::<u32>()?;
    Ok(msg)
}
pub async fn run(id: u32, mut rx: mpsc::Receiver<Json<Message>>, tx: mpsc::Sender<Action>) {
    let conn = Conn::new().await;
    loop {
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(msg)) => match conn.send(msg.id, &Message::new(id, msg.data.clone())).await {
                Ok(_) => {}
                Err(e) => {
                    #[cfg(debug_assertions)]
                    tx.send(Action::Err(e.to_string())).await.unwrap();
                }
            },
            Ok(None) => {
                #[cfg(debug_assertions)]
                tx.send(Action::Err("recv channel closed".to_string()))
                    .await
                    .unwrap();
            }
            Err(_) => {
                // tx.send(Action::Err(e.to_string())).await.unwrap();
            }
        }
        match conn.recv(id).await {
            Ok(msgs) => {
                for msg in msgs {
                    tx.send(Action::Receive(msg)).await.unwrap();
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                tx.send(Action::Err(e.to_string())).await.unwrap();
            }
        }
    }
}
