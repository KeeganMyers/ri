use crate::{token::{display_token::*, command_token::*,normal_token::*, Token}};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crate::Window;
use std::sync::{Mutex,Arc};
use ropey::Rope;
use std::collections::HashMap;
use std::io::{stdout};
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use syntect::util::LinesWithEndings;
use crossterm::terminal::ClearType;
use crossterm::{event,  terminal}; 
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use crossterm::{
    cursor::position,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use std::io::Stdout;
use actix::prelude::*;

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Wrap},
    Frame, Terminal,
};
use uuid::Uuid;
pub type Term = Terminal<CrosstermBackend<Stdout>>;

#[derive(Default)]
pub struct Ui {
    pub app: Option<Recipient<Token>>,
    pub should_quit: bool,
    pub syntax_set: Option<SyntaxSet>,
    pub theme_set: Option<ThemeSet>,
    pub theme: Option<Theme>,
    pub syntax: Option<SyntaxReference>,
    pub current_window_id: Uuid,
    pub head_area: Rect,
    pub text_area: Rect,
    pub foot_area: Rect,
    pub terminal: Option<Arc<Mutex<Term>>>
    //pub highlight_cache: HashMap<Uuid, Vec<Spans<'a>>>,
    //pub line_num_cache: HashMap<Uuid, Spans<'a>>,
}


impl Actor for Ui {
    type Context = Context<Self>;
}

impl Ui {
    pub fn convert_style(style: SyntectStyle) -> Style {
        Style::default().fg(Color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b,
        ))
    }

    pub fn to_span(style: SyntectStyle, value: &str) -> Span {
        Span::styled(value, Self::convert_style(style))
    }

    pub fn to_spans(highlights: Vec<(SyntectStyle, &str)>) -> Spans {
        Spans::from(
            highlights
                .iter()
                .map(|h| Self::to_span(h.0, h.1))
                .collect::<Vec<Span>>(),
        )
    }

    pub fn line_number_spans(line_number_count: usize) -> Spans<'static> {
        Spans::from(
            (1..line_number_count)
                .map(|l| Span::styled(format!("{:<5}", l), Style::default().fg(Color::Yellow)))
                .collect::<Vec<Span>>(),
        )
    }

    pub fn add_text_split(&mut self, direction: Direction) {
        /*
        let text_area = if let Some(area) = self
            .windows
            .get(&self.current_window_id)
            .and_then(|w| w.area)
        {
            area
        } else {
            self.text_area
        };
        let text_splits = Layout::default()
            .direction(direction)
            .constraints(
                self.windows
                    .values()
                    .map(|_w| Constraint::Percentage(50))
                    .collect::<Vec<Constraint>>()
                    .as_ref(),
            )
            .split(text_area);
        let mut sorted_windows = self
            .windows
            .values()
            .map(|w| w.clone())
            .collect::<Vec<Window>>();
        sorted_windows.sort_by(|w_a, w_b| w_a.get_origin().cmp(&w_b.get_origin()));

        for (window_id, split) in sorted_windows.into_iter().map(|w| w.id).zip(text_splits) {
            if let Some(window) = self.windows.get_mut(&window_id) {
                window.outer_areas.push(window.area);
                window.area = Some(split);
                window.bottom = Some(split.bottom());
                window.right = Some(split.right());
            }
        }
        */
    }

    pub fn remove_text_split(&mut self, window_id: Uuid) {
        /*
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.area = window.outer_areas.pop().flatten();
            window.bottom = window.area.map(|a| a.bottom());
            window.right = window.area.map(|a| a.right());
        }
        */
    }

    pub fn draw<'a,B: 'a> (head_area: Rect,foot_area: Rect, text_area: Rect,window_widgets: Vec<Window>,f: &mut Frame<'a,B>,) 
        where 
            B: Backend
    {
        //let current_window = windows.get(&current_window_id).cloned();
        Self::draw_header(
            None,
            f,
            head_area,
        );

        for window in window_widgets {
            f.render_widget(window,text_area)
        }

        /*
        for window in windows.values() {
             f.render_widget(window.clone(), f.size());

            if window.id == current_window_id {
                let y_cursor = if window.display_y_pos() >= window.bottom.unwrap_or_default() {
                    window.bottom.map(|b| b - 3).unwrap_or_default()
                } else {
                    window.display_y_pos()
                };

                let x_cursor = if window.display_x_pos() >= window.right.unwrap_or_default() {
                    window.right.map(|r| r - 1).unwrap_or_default()
                } else {
                    window.display_x_pos()
                };

                f.set_cursor(x_cursor, y_cursor);
            }
        }
        */
        Self::draw_footer(
            None,
            f,
            foot_area,
        );
    }

    fn create_layout<B: Backend>(frame: &Frame<B>) -> (Rect, Rect, Rect) {
        let area = Layout::default()
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Min(20),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(frame.size());
        (area[0], area[1], area[2])
    }

    async fn cache_formatted_text(&mut self, text: &Rope, id: Uuid) {
        if let (Some(syntax), Some(theme)) = (&self.syntax, &self.theme) {
            let mut highlight = HighlightLines::new(syntax, theme);
            let mut spans: Vec<Spans> = vec![];
            let rope_str = text.to_string();
            let text_lines = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            for line in text_lines {
                if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                    spans.push(Self::to_spans(hs.clone()));
                }
            }
            unimplemented!();
            //self.highlight_cache.insert(id, spans.clone());
        }
    }

    async fn cache_new_line(&mut self, text: &Rope, id: Uuid, line_index: usize) {
        if let (Some(syntax), Some(theme)) = (&self.syntax, &self.theme) {
            let mut highlight = HighlightLines::new(syntax, theme);
            let rope_str = text
                .get_line(line_index)
                .map(|l| l.to_string())
                .unwrap_or_default();
            let mut text_line = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            if let Some(line) = &text_line.nth(0) {
            unimplemented!();
                /*
                if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                    if let Some(cache) = self.highlight_cache.get_mut(&id) {
                        cache.insert(line_index, Self::to_spans(hs.clone()));
                    }
                }
                */
            }
        }
    }

    async fn cache_current_line(&mut self, text: &Rope, id: Uuid, line_index: usize) {
        if let (Some(syntax), Some(theme)) = (&self.syntax, &self.theme) {
            let mut highlight = HighlightLines::new(syntax, theme);
            let rope_str = text
                .get_line(line_index)
                .map(|l| l.to_string())
                .unwrap_or_default();
            let mut text_line = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            if let Some(line) = &text_line.nth(0) {
                if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                    unimplemented!();
                    /*
                    if let Some(cache) = self.highlight_cache.get_mut(&id) {
                        cache[line_index] = Self::to_spans(hs.clone());
                    }
                    */
                }
            }
        }
    }

    async fn remove_cache_line(&mut self, id: Uuid, line_index: usize) {
        unimplemented!();
        /*
        if let Some(cache) = self.highlight_cache.get_mut(&id) {
            cache.remove(line_index);
        }
        */
    }

    async fn cache_line_numbers(&mut self, text: &Rope, id: Uuid) {
        let line_count = text.len_lines();
        let local_line_nums = Self::line_number_spans(line_count);
        unimplemented!();
        //self.line_num_cache.insert(id, local_line_nums.clone());
    }


    fn draw_footer<B>(window: Option<Window>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph = Paragraph::new(
            window.clone()
                .and_then(|w| w.command_text.clone())
                .unwrap_or("".to_string()),
        )
        .block(block.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
        let paragraph2 = Paragraph::new(format!(
            "{:?}",
            window.clone().map(|w| w.mode.clone()).unwrap_or_default()
        ))
        .block(block.clone())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        let paragraph3 = Paragraph::new(format!(
            "{},{}",
            window.clone().map(|w| w.y_pos).unwrap_or_default(),
            window.map(|w| w.x_pos).unwrap_or_default()
        ))
        .block(block)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        f.render_widget(paragraph2, area);
        f.render_widget(paragraph3, area);
    }

    fn draw_header<B>(window: Option<Window>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph =
            Paragraph::new(window.map(|w| w.title.clone()).unwrap_or_default()).block(block);
        f.render_widget(paragraph, area);
    }

    pub fn new(app: Recipient<Token>,terminal: Arc<Mutex<Term>>) -> AnyHowResult<Self> {
      /*
        let _ = execute!(stdout(), terminal::Clear(ClearType::All));
       let mut stdout = stdout();
        execute!(stdout, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
      */
        let  term_arc = terminal.clone();
        let mut term_lock = term_arc.lock().map_err(|e| AnyHowError::msg(format!("{:?}",e)))?;
        let (head_area, text_area, foot_area) = Ui::create_layout(&term_lock.get_frame());
        let ui = Self {
            app: Some(app),
            should_quit: false,
            syntax_set: None,
            theme_set: None,
            theme: None,
            syntax: None,
            current_window_id: Uuid::new_v4(),
            head_area,
            text_area,
            foot_area,
            terminal: Some(terminal.clone()),
            ..Self::default()
        };
        Ok(ui)
    }

    fn handle_display_token(
        &mut self,
        token: DisplayToken,
    ) -> AnyHowResult<Vec<Token>> {
        match token {
            DisplayToken::DrawViewPort(window_widgets) => {
                let head_area = self.head_area.clone();
                let foot_area = self.foot_area.clone();
                let text_area = self.text_area.clone();
                let current_window_id = self.current_window_id.clone();
                if let Some(mut term) = self.terminal.as_ref().and_then(|t| t.lock().ok()) {
                    let _ = term.draw(|f| Self::draw(head_area,foot_area,text_area,window_widgets,f));
                }
            }
            DisplayToken::SetHighlight => {
                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let syntax = ps.find_syntax_by_extension("rs").clone();
                let theme = ts.themes["base16-ocean.dark"].clone();
                self.syntax_set = Some(ps.clone());
                self.theme_set = Some(ts);
                if let Some(s) = syntax {
                    self.syntax = Some(s.clone());
                }
                self.theme = Some(theme);
            }
            DisplayToken::NewWindow(change, direction) => {
                /*
                let right_offset = self
                    .windows
                    .get(&change.id)
                    .and_then(|w| w.window_left)
                    .and_then(|w| self.windows.get(&w))
                    .and_then(|w| w.right)
                    .unwrap_or_default();
                let top_offset = self
                    .windows
                    .get(&change.id)
                    .and_then(|w| w.window_up)
                    .and_then(|w| self.windows.get(&w))
                    .and_then(|w| w.bottom)
                    .unwrap_or(self.text_area.top());
                let mut window = Window {
                    id: change.id,
                    buffer_id: change.id,
                    x_pos: change.x_pos,
                    y_pos: change.y_pos,
                    mode: change.mode,
                    title: change.title.unwrap_or_default(),
                    page_size: change.page_size,
                    current_page: change.current_page,
                    y_offset: top_offset,
                    x_offset: right_offset + 4,
                    ..Window::default()
                };

                match direction {
                    Some(Direction::Horizontal) => {
                        window.window_right = Some(self.current_window_id);
                        if let Some(current_window) = self.windows.get_mut(&self.current_window_id)
                        {
                            current_window.window_left = Some(window.id);
                        }
                    }
                    Some(Direction::Vertical) => {
                        window.window_down = Some(self.current_window_id);
                        if let Some(current_window) = self.windows.get_mut(&self.current_window_id)
                        {
                            current_window.window_up = Some(window.id);
                        }
                    }
                    None => (),
                }

                self.windows.insert(change.id, window);
                self.current_window_id = change.id;
                */
            }
            DisplayToken::SetTextLayout(direction) => {
                self.add_text_split(direction);
            }
            DisplayToken::UpdateWindow(change) => {
                /*
                let right_offset = self
                    .windows
                    .get(&change.id)
                    .and_then(|w| w.window_left)
                    .and_then(|w| self.windows.get(&w))
                    .and_then(|w| w.right)
                    .unwrap_or_default();
                let top_offset = self
                    .windows
                    .get(&change.id)
                    .and_then(|w| w.window_up)
                    .and_then(|w| self.windows.get(&w))
                    .and_then(|w| w.bottom)
                    .unwrap_or(self.text_area.top());
                if let Some(window) = self.windows.get_mut(&change.id) {
                    window.x_pos = change.x_pos;
                    window.y_pos = change.y_pos;
                    window.mode = change.mode;
                    window.title = change.title.unwrap_or_default();
                    window.page_size = change.page_size;
                    window.current_page = change.current_page;
                    window.y_offset = top_offset;
                    window.x_offset = 4 + right_offset;
                }
                */
            }
            DisplayToken::CacheWindowContent(id, text) => {
                //self.cache_formatted_text(&text, id).await;
                //self.cache_line_numbers(&text, id).await;
            }
            DisplayToken::CloseWindow(id) => {
                /*
                let current_windows = self.windows.clone();
                let current_window = current_windows.get(&id);
                if let Some(current_window) = current_window {
                    self.windows.remove(&id);
                    //self.highlight_cache.remove(&id);
                    //self.line_num_cache.remove(&id);
                    match current_window.clone() {
                        Window {
                            window_right: Some(window_right),
                            window_left,
                            ..
                        } => {
                            self.current_window_id = window_right;
                            self.windows
                                .get_mut(&window_right)
                                .map(|w| w.window_left = window_left);
                            self.remove_text_split(window_right);
                        }
                        Window {
                            window_left: Some(window_left),
                            window_right,
                            ..
                        } => {
                            self.current_window_id = window_left;
                            self.windows
                                .get_mut(&window_left)
                                .map(|w| w.window_right = window_right);
                            self.remove_text_split(window_left);
                        }
                        Window {
                            window_up: Some(window_up),
                            window_down,
                            ..
                        } => {
                            self.current_window_id = window_up;
                            self.windows
                                .get_mut(&window_up)
                                .map(|w| w.window_down = window_down);
                            self.remove_text_split(window_up);
                        }
                        Window {
                            window_down: Some(window_down),
                            window_up,
                            ..
                        } => {
                            self.current_window_id = window_down;
                            self.windows
                                .get_mut(&window_down)
                                .map(|w| w.window_up = window_up);
                            self.remove_text_split(window_down);
                        }
                        _ => (),
                    }
                }
                */
            }
            DisplayToken::CacheCurrentLine(id, text, line_index) => {
                //self.cache_current_line(&text, id, line_index).await;
            }
            DisplayToken::CacheNewLine(id, text, line_index) => {
                //self.cache_new_line(&text, id, line_index).await;
                //self.cache_line_numbers(&text, id).await;
            }
            DisplayToken::RemoveCacheLine(id, text, line_index) => {
                //self.remove_cache_line(id, line_index).await;
                //self.cache_line_numbers(&text, id).await;
            }
            DisplayToken::AppendCommand(id, command) => {
                /*
                if let Some(window) = self.windows.get_mut(&id) {
                    window.command_text = command;
                }
                */
            }
            _ => (),
        };

        Ok(vec![])
    }

    async fn handle_command_token(
        &mut self,
       // _terminal: &mut Term,
        token: CommandToken,
    ) -> AnyHowResult<Vec<Token>> {
        let _ = match token {
            CommandToken::Quit => {
            self.should_quit = true;
                Ok(())
            }
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        };
        Ok(vec![])
    }

    async fn handle_normal_token(
        &mut self,
        //terminal: &mut Term,
        token: NormalToken,
    ) -> AnyHowResult<Vec<Token>> {
        match token {
            NormalToken::WindowLeft => {
                /*
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_left)
                {
                    /*
                    if let Some(target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        let _ = terminal.draw(|f| self.draw(f));
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                    }
                    */
                }
            */
                Ok(vec![])
            }
            NormalToken::WindowRight => {
                /*
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_right)
                {
                    /*
                    if let Some(target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        let _ = terminal.draw(|f| self.draw(f));
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                    }
                    */
                }
            */
                Ok(vec![])
            }
            NormalToken::WindowUp => {
                /*
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_up)
                {
                    if let Some(target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                    }
                }
                */
                Ok(vec![])
            }
            NormalToken::WindowDown => {
                /*
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_down)
                {
                    if let Some(target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                    }
                }
                */
                Ok(vec![])
            }
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        }
    }
}

    impl Handler<Token> for Ui {
        type Result = ();

        fn handle(&mut self, msg: Token , _ctx: &mut Context<Self>) -> Self::Result {
            match msg {
                Token::Display(t) => {
                    self.handle_display_token(t);
                    ()
                },
                /*
                Token::Command(t) => {
                    self.handle_command_token(&mut self.terminal, t);
                    ()
                },
                Token::Normal(t) => {
                    self.handle_normal_token(&mut self.terminal, t);
                    ()
                },
                */
                _ => (),
            }
            ()
        }
    }
