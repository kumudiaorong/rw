use chat::Message;
use rocket::serde::json::Json;
use rocket::{get, post, routes};
use rocket::{launch, State};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

struct TmpMessage {
    group_chat: Vec<Arc<Message>>,
    chat: Vec<Message>,
}
impl TmpMessage {
    pub fn new() -> Self {
        Self {
            group_chat: vec![],
            chat: vec![],
        }
    }
    pub fn push(&mut self, msg: Message) {
        self.chat.push(msg);
    }
    pub fn push_group(&mut self, msg: Arc<Message>) {
        self.group_chat.push(msg);
    }
    pub fn extract(&mut self) -> Vec<Message> {
        let mut msgs = self.chat.drain(..).collect::<Vec<_>>();
        msgs.extend(self.group_chat.drain(..).map(|v| v.as_ref().clone()));
        msgs
    }
}
struct Server {
    last_id: AtomicU32,
    // msg: std::collections::HashMap<u32, Massage>,
    msg: RwLock<std::collections::HashMap<u32, TmpMessage>>,
}

#[get("/")]
async fn index(state: &State<Server>) -> String {
    let count = state.last_id.fetch_add(1, Ordering::Relaxed);
    count.to_string()
}

#[post("/send?<dst>", format = "json", data = "<msg>")]
async fn send(dst: u32, msg: Json<Message>, state: &State<Server>) {
    let msg = msg.into_inner();
    if dst == 1 {
        let src = msg.id;
        let msg = Arc::new(Message::new(1, msg.data));
        state.msg.write().await.iter_mut().for_each(|(id, tm)| {
            if *id != src {
                tm.push_group(msg.clone());
            }
        });
    } else {
        println!("{}: {}", dst, msg.to_string());
        state
            .msg
            .write()
            .await
            .entry(dst)
            .or_insert(TmpMessage::new())
            .push(msg);
    }
}
#[get("/recv?<dst>")]
async fn recv(dst: u32, state: &State<Server>) -> Json<Vec<Message>> {
    Json::from(
        state
            .msg
            .write()
            .await
            .entry(dst)
            .or_insert(TmpMessage::new())
            .extract(),
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::config::Config::figment().merge(("log_level", "off")))
        .mount("/", routes![index, send, recv])
        .manage(Server {
            last_id: AtomicU32::new(2),
            msg: RwLock::new(std::collections::HashMap::new()),
        })
}
