mod action;
mod conn;
mod ui;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::error::Error;

pub async fn run() -> Result<(), Box<dyn Error>> {
    // setup termina
    let id = conn::new().await?;
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let (tx1, rx1) = tokio::sync::mpsc::channel(32);
    tokio::spawn(async move {
        ui::Ui::new(id, rx, tx1).run().await;
    });
    let usetx = tx.clone();
    tokio::spawn(conn::run(id, rx1, usetx));
    // create app and run it
    // ?;
    loop {
        // ?;
        let event = event::read().unwrap();
        // ?;
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Esc => {
                        tx.send(action::Action::Over).await.unwrap();
                        break;
                    }
                    _ => {
                        tx.send(action::Action::Event(event)).await.unwrap();
                    }
                }
            }
        }
    }
    // restore terminal

    Ok(())
}
