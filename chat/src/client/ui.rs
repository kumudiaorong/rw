use super::action::Action;
use crate::{Data, Message};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    Frame,
    {prelude::*, widgets::*},
};
use rocket::serde::json::Json;
use std::{
    collections::{BTreeMap, HashMap},
    io::{stdout, Write},
};
use tokio::sync::mpsc as tokio_mpsc;
use tui_input::{
    backend::crossterm::{write as input_write, EventHandler},
    Input,
};

#[derive(Clone, Debug)]
struct Record {
    time_stamp: u32,
    data: Vec<(bool, String)>,
}
impl Record {
    pub fn new(time_stamp: u32) -> Self {
        Self {
            time_stamp,
            data: vec![],
        }
    }
    pub fn push(&mut self, data: String) {
        self.data.push((true, data));
    }
    pub fn push_self(&mut self, data: String) {
        self.data.push((false, data));
    }
}

pub struct Ui {
    state: State,
    pub rx: tokio_mpsc::Receiver<Action>,
    pub tx: tokio_mpsc::Sender<Json<Message>>,
}
impl Ui {
    pub fn new(
        id: u32,
        rx: tokio_mpsc::Receiver<Action>,
        tx: tokio_mpsc::Sender<Json<Message>>,
    ) -> Self {
        Self {
            state: State::new(id),
            rx,
            tx,
        }
    }
    pub async fn run(&mut self) {
        enable_raw_mode().expect("can run in raw mode");
        // ?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture).expect("can run in raw mode");
        // ?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend).unwrap();
        // ?;
        loop {
            terminal.draw(|f| ui(f, &self.state)).expect("can draw");
            match self.rx.recv().await.expect("can recv") {
                Action::Receive(msg) => {
                    self.state.err = format!("recv {:?}", msg.to_string());
                    self.state.list.update(msg.id).push(msg.to_string());
                    if let Data::File { filename, file } = msg.data {
                        tokio::fs::write(filename, file).await.expect("can write");
                    }
                }
                Action::Event(event) => {
                    if let Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code,
                        ..
                    }) = event
                    {
                        match code {
                            KeyCode::Enter => {
                                let str = self.state.input.value().to_string();
                                if str.starts_with("\\") {
                                    if str.starts_with("\\f:") {
                                        if let Ok(content) =
                                            tokio::fs::read_to_string(&str[3..]).await
                                        {
                                            let msg = Message::new(
                                                self.state.selected,
                                                Data::File {
                                                    filename: str[3..]
                                                        .rsplit_once('/')
                                                        .unwrap_or(("", &str[3..]))
                                                        .1
                                                        .to_string(),
                                                    file: content.into_bytes(),
                                                },
                                            );
                                            self.tx.send(Json::from(msg)).await.expect("can send");
                                        }
                                    } else {
                                        let _ = str[1..].parse::<u32>().map(|id| {
                                            self.state.list.update(id);
                                            self.state.selected = id;
                                        });
                                    }
                                } else if self.state.selected != 0 {
                                    self.state
                                        .list
                                        .update(self.state.selected)
                                        .push_self(str.clone());
                                    let msg =
                                        Message::new(self.state.selected, Data::Text(str.clone()));
                                    self.tx.send(Json::from(msg)).await.expect("can send");
                                }
                                self.state.input.reset();
                            }
                            KeyCode::Down => {
                                self.state.selected = self.state.list.next(self.state.selected);
                            }
                            KeyCode::Up => {
                                self.state.selected = self.state.list.previous(self.state.selected);
                            }
                            _ => {
                                if self.state.input.handle_event(&event).is_some() {
                                    stdout().flush().expect("can flush");
                                }
                            }
                        }
                    }
                }
                Action::Over => {
                    disable_raw_mode().expect("can run in raw mode");
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )
                    .expect("can run in raw mode");
                    terminal.show_cursor().expect("can run in raw mode");
                    // ?;
                    break;
                }
                #[cfg(debug_assertions)]
                Action::Err(err) => {
                    self.state.err = format!("net {:?}", err);
                }
            };
        }
    }
}

struct State {
    pub id: u32,
    pub list: LazyList,
    pub selected: u32,
    pub input: Input,
    pub err: String,
}

struct LazyList {
    pub time_stamp: u32,
    pub rank: BTreeMap<u32, u32>,
    pub by_id: HashMap<u32, Record>,
}
impl LazyList {
    pub fn new() -> Self {
        Self {
            time_stamp: 0,
            rank: BTreeMap::new(),
            by_id: HashMap::from([(0, Record::new(0))]),
        }
    }
    pub fn next(&mut self, id: u32) -> u32 {
        let record = self.by_id.get(&id).unwrap();
        let (_, id) = self
            .rank
            .range(..record.time_stamp)
            .last()
            .unwrap_or(self.rank.last_key_value().unwrap_or((&0, &0)));
        *id
    }
    pub fn previous(&mut self, id: u32) -> u32 {
        let record = self.by_id.get(&id).unwrap();
        let (_, id) = self
            .rank
            .range((record.time_stamp + 1)..)
            .next()
            .unwrap_or(self.rank.first_key_value().unwrap_or((&0, &0)));
        *id
    }
    pub fn update(&mut self, id: u32) -> &mut Record {
        self.time_stamp = self.time_stamp + 1;
        match self.by_id.get_mut(&id) {
            Some(record) => {
                self.rank.remove(&record.time_stamp);
                record.time_stamp = self.time_stamp;
            }
            None => {}
        }
        self.rank.insert(self.time_stamp, id);
        self.by_id.entry(id).or_insert(Record::new(self.time_stamp))
    }
}
impl State {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            list: LazyList::new(),
            selected: 0,
            input: Input::new("".to_string()),
            err: "".to_string(),
        }
    }
}

fn ui(f: &mut Frame, app: &State) {
    #[allow(unused_mut)]
    let mut size = f.size();
    #[cfg(debug_assertions)]
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(size);
        let err = Paragraph::new(Line::from(vec!["Error".red(), app.err.clone().gray()]))
            .style(Style::default().fg(Color::Red));
        f.render_widget(err, chunks[1]);

        size = chunks[0];
    }
    let block = Block::default().on_white().black();
    f.render_widget(block.clone(), size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(size);
    let tabs: Vec<ListItem> = app
        .list
        .rank
        .iter()
        .rev()
        .map(|(_, id)| {
            let id = if *id == app.selected {
                id.to_string().red()
            } else {
                id.to_string().green()
            };
            ListItem::new(id)
        })
        .collect();
    let list = List::new(tabs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Who")
                .border_type(BorderType::Rounded),
        )
        .style(Style::default().cyan().on_gray());
    f.render_widget(list, chunks[0]);
    let sub_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(chunks[1]);
    let cells = if app.selected == 0 {
        vec![]
    } else {
        let data = &app.list.by_id.get(&app.selected).unwrap().data;
        let max = sub_chunks[0].height.saturating_sub(2) as usize;
        data.iter()
            .skip(data.len().saturating_sub(max))
            .map(|(other, str)| {
                Row::new(vec![Line::from(str.clone().green()).alignment(if *other {
                    Alignment::Left
                } else {
                    Alignment::Right
                })])
            })
            .collect::<Vec<_>>()
    };

    let chat = Table::new(cells)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("[{:03}]", app.selected))
                .border_type(BorderType::Rounded),
        )
        .widths(&[Constraint::Percentage(100)]);
    f.render_widget(chat, sub_chunks[0]);

    let _ = input_write(
        &mut stdout(),
        app.input.value(),
        app.input.cursor(),
        (sub_chunks[1].x + 5, sub_chunks[1].y + 1),
        sub_chunks[1].width.saturating_sub(6),
    );
    let input = Paragraph::new(format!("{:03}>", app.id,)).block(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    f.render_widget(input, sub_chunks[1]);
}
