pub enum Action {
    Receive(crate::Message),
    Event(crossterm::event::Event),
    Over,
    #[cfg(debug_assertions)]
    Err(String),
}
