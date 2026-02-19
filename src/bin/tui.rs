use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use sipp_rust::db::{self, Snippet};
use std::time::{Duration, Instant};
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use std::io::Cursor;

enum Focus {
    List,
    Content,
}

struct App {
    snippets: Vec<Snippet>,
    list_state: ListState,
    should_quit: bool,
    status_message: Option<(String, Instant)>,
    focus: Focus,
    content_scroll: u16,
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl App {
    fn new(snippets: Vec<Snippet>) -> Self {
        let mut list_state = ListState::default();
        if !snippets.is_empty() {
            list_state.select(Some(0));
        }
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_data = include_bytes!("../ansi.tmTheme");
        let theme =
            syntect::highlighting::ThemeSet::load_from_reader(&mut Cursor::new(&theme_data[..]))
                .expect("failed to load base16 theme");
        Self {
            snippets,
            list_state,
            should_quit: false,
            status_message: None,
            focus: Focus::List,
            content_scroll: 0,
            syntax_set,
            theme,
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
        self.content_scroll = 0;
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
        self.content_scroll = 0;
    }

    fn scroll_up(&mut self) {
        self.content_scroll = self.content_scroll.saturating_sub(1);
    }

    fn scroll_down(&mut self, max_lines: u16) {
        if self.content_scroll < max_lines {
            self.content_scroll += 1;
        }
    }

    fn copy_selected(&mut self) {
        if let Some(snippet) = self.selected_snippet() {
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(&snippet.content);
                self.status_message = Some(("Copied!".to_string(), Instant::now()));
            }
        }
    }

    fn clear_expired_status(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed() > Duration::from_secs(2) {
                self.status_message = None;
            }
        }
    }

    fn highlight_content(&self, name: &str, content: &str) -> Text<'static> {
        let ext = name.rsplit('.').next().unwrap_or("");
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(ext)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);

        let lines: Vec<Line<'static>> = LinesWithEndings::from(content)
            .map(|line| {
                let ranges = highlighter
                    .highlight_line(line, &self.syntax_set)
                    .unwrap_or_default();
                let spans: Vec<Span<'static>> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let color = to_ratatui_color(style.foreground);
                        Span::styled(text.to_owned(), Style::default().fg(color))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect();

        Text::from(lines)
    }
}

fn to_ratatui_color(color: syntect::highlighting::Color) -> Color {
    if color.a == 0 {
        Color::Indexed(color.r)
    } else {
        Color::Reset
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
        app.clear_expired_status();

        let content_line_count = app
            .selected_snippet()
            .map(|s| s.content.lines().count() as u16)
            .unwrap_or(0);

        terminal.draw(|frame| {
            let outer = Layout::vertical([
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(frame.area());

            let chunks = Layout::horizontal([
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(outer[0]);

            let items: Vec<ListItem> = app
                .snippets
                .iter()
                .map(|s| ListItem::new(s.name.as_str()))
                .collect();

            let list_border_style = match app.focus {
                Focus::List => Style::default().fg(Color::Yellow),
                Focus::Content => Style::default().fg(Color::DarkGray),
            };
            let content_border_style = match app.focus {
                Focus::Content => Style::default().fg(Color::Yellow),
                Focus::List => Style::default().fg(Color::DarkGray),
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" Snippets ")
                        .borders(Borders::ALL)
                        .border_style(list_border_style),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(list, chunks[0], &mut app.list_state);

            let highlighted = match app.selected_snippet() {
                Some(s) => app.highlight_content(&s.name, &s.content),
                None => Text::raw(""),
            };

            let paragraph = Paragraph::new(highlighted)
                .block(
                    Block::default()
                        .title(" Content ")
                        .borders(Borders::ALL)
                        .border_style(content_border_style),
                )
                .scroll((app.content_scroll, 0));

            frame.render_widget(paragraph, chunks[1]);

            if let Some((msg, _)) = &app.status_message {
                let status = Paragraph::new(Text::raw(msg.as_str()))
                    .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
                frame.render_widget(status, outer[1]);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.focus {
                    Focus::List => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                        KeyCode::Char('y') => app.copy_selected(),
                        KeyCode::Enter => {
                            if app.selected_snippet().is_some() {
                                app.focus = Focus::Content;
                            }
                        }
                        _ => {}
                    },
                    Focus::Content => match key.code {
                        KeyCode::Char(' ') | KeyCode::Esc | KeyCode::Char('q') => {
                            app.focus = Focus::List;
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.scroll_down(content_line_count);
                        }
                        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
                        KeyCode::Char('y') => app.copy_selected(),
                        _ => {}
                    },
                }
            }
        }
    }

    Ok(())
}
