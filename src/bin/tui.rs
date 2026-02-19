use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use sipp_rust::db::{self, Snippet};

struct App {
    snippets: Vec<Snippet>,
    list_state: ListState,
    should_quit: bool,
}

impl App {
    fn new(snippets: Vec<Snippet>) -> Self {
        let mut list_state = ListState::default();
        if !snippets.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            snippets,
            list_state,
            should_quit: false,
        }
    }

    fn selected_snippet(&self) -> Option<&Snippet> {
        self.list_state.selected().and_then(|i| self.snippets.get(i))
    }

    fn move_up(&mut self) {
        if self.snippets.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => self.snippets.len() - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn move_down(&mut self) {
        if self.snippets.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i < self.snippets.len() - 1 => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn copy_selected(&self) {
        if let Some(snippet) = self.selected_snippet() {
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(&snippet.content);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db();
    let snippets = db::get_all_snippets(&db);

    ratatui::run(|terminal| run_app(terminal, App::new(snippets)))
}

fn run_app(
    terminal: &mut DefaultTerminal,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        terminal.draw(|frame| {
            let chunks = Layout::horizontal([
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(frame.area());

            let items: Vec<ListItem> = app
                .snippets
                .iter()
                .map(|s| ListItem::new(s.name.as_str()))
                .collect();

            let list = List::new(items)
                .block(Block::default().title(" Snippets ").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

            let content = app
                .selected_snippet()
                .map(|s| s.content.as_str())
                .unwrap_or("");

            let paragraph = Paragraph::new(Text::raw(content))
                .block(Block::default().title(" Content ").borders(Borders::ALL));

            frame.render_widget(paragraph, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                KeyCode::Char('y') => app.copy_selected(),
                _ => {}
            }
        }
    }

    Ok(())
}
