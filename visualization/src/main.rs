mod algorithms;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use std::io::{self, stdout};
#[derive(Debug)]
pub enum State {
    Start,
    Sorted,
    Selected,
    Swapped,
    Repeated,
    Done,
}
#[derive(Debug)]
struct Action {
    all: Vec<char>,
    swap_idx: usize,
    selected: usize,
    state: State,
}
impl Action {
    fn new(all: Vec<char>, swap_idx: usize, selected: usize, state: State) -> Self {
        Self {
            all,
            swap_idx,
            selected,
            state,
        }
    }
}

struct App {
    actions: Vec<Action>,
    idx: usize,
    skip: usize,
    #[cfg(debug_assertions)]
    error: String,
}
impl App {
    fn new(actions: Vec<Action>) -> Self {
        Self {
            actions,
            idx: 0,
            skip: 0,
            #[cfg(debug_assertions)]
            error: String::new(),
        }
    }
    fn selected(&self) -> &Action {
        &self.actions[self.idx]
    }
    fn next(&mut self) {
        self.idx = (self.idx + 1).min(self.actions.len() - 1);
    }
    fn prev(&mut self) {
        self.idx = self.idx.saturating_sub(1);
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut should_quit = false;
    let mut actions = Vec::new();
    algorithms::run(&mut |all, swap_idx, selected, state| {
        actions.push(Action::new(all.clone(), swap_idx, selected, state));
    });
    let mut app = App::new(actions);
    while !should_quit {
        terminal.draw(|f| ui(f, &app))?;
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => should_quit = true,
                        KeyCode::Down => {
                            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.skip += 1;
                                #[cfg(debug_assertions)]
                                {
                                    app.error = format!("skip: {}", app.skip);
                                }
                            } else {
                                app.next();
                            }
                        }
                        KeyCode::Up => {
                            if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                app.skip = app.skip.saturating_sub(1);
                            } else {
                                app.prev();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    #[allow(unused_mut)]
    let mut size = frame.size();
    #[cfg(debug_assertions)]
    {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(size);
        frame.render_widget(Paragraph::new(format!("{:?}", app.error)), chunks[1]);
        size = chunks[0];
    }
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(app.selected().all.len() as u16 + 2),
        ])
        .split(size);

    let block = Block::default().title("All").borders(Borders::ALL);
    let all: Vec<_> = app
        .actions
        .iter()
        .enumerate()
        .filter_map(|(i, a)| match a.state {
            State::Done if i < app.idx => Some(Line::from(a.all.iter().collect::<String>())),
            _ => None,
        })
        .collect();
    let alllen = all.len();
    let all = all
        .into_iter()
        .skip(
            app.skip
                .min(alllen.saturating_sub(chunks[1].height as usize - 2)) as usize,
        )
        .take(chunks[1].height as usize - 2)
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(all).block(block);
    frame.render_widget(paragraph, chunks[1]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(chunks[0]);
    let block = Block::default().title("Actions").borders(Borders::ALL);
    let t: Vec<_> = app
        .selected()
        .all
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let style = if i < app.selected().selected {
                Style::default().fg(Color::Green)
            } else if i == app.selected().selected || i == app.selected().swap_idx {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };
            c.to_string().set_style(style)
        })
        .collect();
    let paragraph = Paragraph::new(Line::from(t)).block(block);
    frame.render_widget(paragraph, chunks[0]);
    let block = Block::default().title("Status").borders(Borders::ALL);
    let paragraph = Paragraph::new(match app.selected().state {
        State::Start => "Start",
        State::Sorted => "Sorted",
        State::Selected => "Selected",
        State::Swapped => "Swapped",
        State::Repeated => "Repeated",
        State::Done => "Done",
    })
    .block(block);
    frame.render_widget(paragraph, chunks[1]);
    let paragraph = Paragraph::new(Line::from(vec![
        "Green".green(),
        ": selected | ".into(),
        "Red".red(),
        ": swap | ".into(),
        "White".white(),
        ": unselected".into(),
    ]))
    .block(Block::default().title("Help").borders(Borders::ALL));
    frame.render_widget(paragraph, chunks[2]);
}
