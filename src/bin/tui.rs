use arboard::Clipboard;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
};
use sipp_rust::backend::Backend;
use sipp_rust::db::Snippet;
use std::io::Cursor;
use std::time::{Duration, Instant};
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

#[derive(Parser)]
#[command(name = "sipp-tui", about = "TUI client for sipp snippets")]
struct Cli {
    /// Remote server URL (e.g. http://localhost:3000)
    #[arg(short, long, env = "SIPP_REMOTE_URL")]
    remote: Option<String>,

    /// API key for authenticated operations
    #[arg(short = 'k', long, env = "SIPP_API_KEY")]
    api_key: Option<String>,
}

enum Focus {
    List,
    Content,
    CreateName,
    CreateContent,
}

struct App {
    snippets: Vec<Snippet>,
    list_state: ListState,
    should_quit: bool,
    status_message: Option<(String, Instant)>,
    focus: Focus,
    content_scroll: u16,
    show_help: bool,
    syntax_set: SyntaxSet,
    theme: Theme,
    create_name: String,
    create_content: String,
    is_remote: bool,
}

impl App {
    fn new(snippets: Vec<Snippet>, is_remote: bool) -> Self {
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
            show_help: false,
            syntax_set,
            theme,
            create_name: String::new(),
            create_content: String::new(),
            is_remote,
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

    fn delete_selected(&mut self, backend: &Backend) {
        if let Some(selected_index) = self.list_state.selected() {
            if let Some(snippet) = self.snippets.get(selected_index) {
                let short_id = snippet.short_id.clone();
                match backend.delete_snippet(&short_id) {
                    Ok(true) => {
                        self.snippets.remove(selected_index);
                        if self.snippets.is_empty() {
                            self.list_state.select(None);
                        } else if selected_index >= self.snippets.len() {
                            self.list_state.select(Some(self.snippets.len() - 1));
                        } else {
                            self.list_state.select(Some(selected_index));
                        }
                        self.status_message = Some(("Deleted!".to_string(), Instant::now()));
                    }
                    Ok(false) => {
                        self.status_message =
                            Some(("Snippet not found".to_string(), Instant::now()));
                    }
                    Err(e) => {
                        self.status_message = Some((e.to_string(), Instant::now()));
                    }
                }
            }
        }
    }

    fn refresh(&mut self, backend: &Backend) {
        match backend.list_snippets() {
            Ok(snippets) => {
                self.snippets = snippets;
                if self.snippets.is_empty() {
                    self.list_state.select(None);
                } else {
                    let idx = self.list_state.selected().unwrap_or(0);
                    if idx >= self.snippets.len() {
                        self.list_state.select(Some(self.snippets.len() - 1));
                    }
                }
                self.status_message = Some(("Refreshed!".to_string(), Instant::now()));
            }
            Err(e) => {
                self.status_message = Some((e.to_string(), Instant::now()));
            }
        }
    }

    fn start_create(&mut self) {
        self.create_name.clear();
        self.create_content.clear();
        self.focus = Focus::CreateName;
    }

    fn save_create(&mut self, backend: &Backend) {
        if self.create_name.trim().is_empty() {
            self.status_message = Some(("Name cannot be empty".to_string(), Instant::now()));
            return;
        }
        match backend.create_snippet(&self.create_name, &self.create_content) {
            Ok(snippet) => {
                self.snippets.insert(0, snippet);
                self.list_state.select(Some(0));
                self.status_message = Some(("Created!".to_string(), Instant::now()));
                self.focus = Focus::List;
                self.create_name.clear();
                self.create_content.clear();
            }
            Err(e) => {
                self.status_message = Some((e.to_string(), Instant::now()));
            }
        }
    }

    fn cancel_create(&mut self) {
        self.create_name.clear();
        self.create_content.clear();
        self.focus = Focus::List;
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
    let cli = Cli::parse();

    let (backend, is_remote) = match cli.remote {
        Some(url) => (Backend::remote(url, cli.api_key), true),
        None => (Backend::local(), false),
    };

    let snippets = match backend.list_snippets() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load snippets: {}", e);
            Vec::new()
        }
    };

    ratatui::run(|terminal| run_app(terminal, App::new(snippets, is_remote), &backend))
}

fn run_app(
    terminal: &mut DefaultTerminal,
    mut app: App,
    backend: &Backend,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app.should_quit {
        app.clear_expired_status();

        let content_line_count = app
            .selected_snippet()
            .map(|s| s.content.lines().count() as u16)
            .unwrap_or(0);

        terminal.draw(|frame| {
            let outer = Layout::vertical([Constraint::Min(1), Constraint::Length(1)])
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
                _ => Style::default().fg(Color::DarkGray),
            };
            let content_border_style = match app.focus {
                Focus::Content => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::DarkGray),
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

            // Right pane: either create form or snippet content
            match app.focus {
                Focus::CreateName | Focus::CreateContent => {
                    let create_block = Block::default()
                        .title(" New Snippet ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow));

                    let inner = create_block.inner(chunks[1]);
                    frame.render_widget(create_block, chunks[1]);

                    let form_layout = Layout::vertical([
                        Constraint::Length(3),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ])
                    .split(inner);

                    let name_style = match app.focus {
                        Focus::CreateName => Style::default().fg(Color::Yellow),
                        _ => Style::default().fg(Color::DarkGray),
                    };
                    let name_input = Paragraph::new(app.create_name.as_str()).block(
                        Block::default()
                            .title(" Name ")
                            .borders(Borders::ALL)
                            .border_style(name_style),
                    );
                    frame.render_widget(name_input, form_layout[0]);

                    let content_style = match app.focus {
                        Focus::CreateContent => Style::default().fg(Color::Yellow),
                        _ => Style::default().fg(Color::DarkGray),
                    };
                    let content_input = Paragraph::new(app.create_content.as_str()).block(
                        Block::default()
                            .title(" Content ")
                            .borders(Borders::ALL)
                            .border_style(content_style),
                    );
                    frame.render_widget(content_input, form_layout[1]);

                    let hint = Paragraph::new(Line::from(vec![
                        Span::styled("Enter", Style::default().fg(Color::Yellow)),
                        Span::raw(match app.focus {
                            Focus::CreateName => " next field  ",
                            _ => " newline  ",
                        }),
                        Span::styled("Ctrl+S", Style::default().fg(Color::Yellow)),
                        Span::raw(" save  "),
                        Span::styled("Esc", Style::default().fg(Color::Yellow)),
                        Span::raw(" cancel"),
                    ]));
                    frame.render_widget(hint, form_layout[2]);
                }
                _ => {
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
                }
            }

            if let Some((msg, _)) = &app.status_message {
                let status = Paragraph::new(Text::raw(msg.as_str()))
                    .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
                frame.render_widget(status, outer[1]);
            }

            if app.show_help {
                let area = frame.area();
                let popup_width = 44u16.min(area.width.saturating_sub(4));
                let popup_height = 18u16.min(area.height.saturating_sub(4));
                let popup_area = ratatui::layout::Rect {
                    x: (area.width.saturating_sub(popup_width)) / 2,
                    y: (area.height.saturating_sub(popup_height)) / 2,
                    width: popup_width,
                    height: popup_height,
                };

                let mut help_lines = vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            "  j/↓  ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Move down / Scroll down"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  k/↑  ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Move up / Scroll up"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  Enter",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("  Focus content pane"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  Esc  ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Back / Quit"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  y    ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Copy snippet"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  d    ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Delete snippet"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  c    ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Create snippet"),
                    ]),
                ];

                if app.is_remote {
                    help_lines.push(Line::from(vec![
                        Span::styled(
                            "  r    ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Refresh snippets"),
                    ]));
                }

                help_lines.extend([
                    Line::from(vec![
                        Span::styled(
                            "  q    ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Quit"),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  ?    ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw("Toggle this help"),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  Press any key to close",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]);

                let help_text = Text::from(help_lines);

                Clear.render(popup_area, frame.buffer_mut());
                let help = Paragraph::new(help_text).block(
                    Block::default()
                        .title(" Keybindings ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
                frame.render_widget(help, popup_area);
            }
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.show_help {
                    app.show_help = false;
                } else {
                    match app.focus {
                        Focus::List => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                            KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                            KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                            KeyCode::Char('y') => app.copy_selected(),
                            KeyCode::Char('d') => app.delete_selected(backend),
                            KeyCode::Char('c') => app.start_create(),
                            KeyCode::Char('r') if app.is_remote => app.refresh(backend),
                            KeyCode::Char('?') => app.show_help = true,
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
                            KeyCode::Char('?') => app.show_help = true,
                            _ => {}
                        },
                        Focus::CreateName => {
                            if key.modifiers.contains(KeyModifiers::CONTROL)
                                && key.code == KeyCode::Char('s')
                            {
                                app.save_create(backend);
                            } else {
                                match key.code {
                                    KeyCode::Esc => app.cancel_create(),
                                    KeyCode::Enter => app.focus = Focus::CreateContent,
                                    KeyCode::Backspace => {
                                        app.create_name.pop();
                                    }
                                    KeyCode::Char(c) => app.create_name.push(c),
                                    _ => {}
                                }
                            }
                        }
                        Focus::CreateContent => {
                            if key.modifiers.contains(KeyModifiers::CONTROL)
                                && key.code == KeyCode::Char('s')
                            {
                                app.save_create(backend);
                            } else {
                                match key.code {
                                    KeyCode::Esc => app.cancel_create(),
                                    KeyCode::Enter => app.create_content.push('\n'),
                                    KeyCode::Backspace => {
                                        app.create_content.pop();
                                    }
                                    KeyCode::Char(c) => app.create_content.push(c),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
