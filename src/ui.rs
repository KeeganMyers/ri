use crate::token::{display_token::*, command_token::*,normal_token::*, Token};
use crate::Window;
use anyhow::{Result as AnyHowResult};
use flume::{Receiver, Sender};
use ropey::Rope;
use std::collections::HashMap;
use std::io::{stdout, Stdout};
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use syntect::util::LinesWithEndings;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use termion::{
    input::MouseTerminal,
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};
use actix::prelude::*;

use tui::{
    backend::{Backend, TermionBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Wrap},
    Frame, Terminal,
};
use uuid::Uuid;
pub type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>;
pub struct Ui{
    pub should_quit: bool,
    pub windows: HashMap<Uuid, Window>,
    pub syntax_set: Option<SyntaxSet>,
    pub theme_set: Option<ThemeSet>,
    pub theme: Option<Theme>,
    pub syntax: Option<SyntaxReference>,
    pub current_window_id: Uuid,
    pub head_area: Rect,
    pub text_area: Rect,
    pub foot_area: Rect,
    //pub highlight_cache: HashMap<Uuid, Vec<Spans<'a>>>,
    //pub line_num_cache: HashMap<Uuid, Spans<'a>>,
}


impl Actor for Ui {
    type Context = Context<Self>;
}

impl Handler<CacheWindowContent> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: CacheWindowContent, _ctx: &mut Context<Self>) {
                unimplemented!();
                //self.cache_formatted_text(&text, msg.id).await;
                //self.cache_line_numbers(&text, msg.id).await;
    }
}

impl Handler<WindowUp> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: WindowUp, _ctx: &mut Context<Self>) {
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_up)
                {
                    if let Some(_target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        unimplemented!();
                        /*
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                        */
                    }
                }
    }
}


impl Handler<WindowDown> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: WindowDown, _ctx: &mut Context<Self>) {
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_down)
                {
                    if let Some(_target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        unimplemented!();
                        /*
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                        */
                    }
                }
    }
}


impl Handler<WindowLeft> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: WindowLeft, _ctx: &mut Context<Self>) {
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_left)
                {
                    if let Some(_target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        unimplemented!();
                        /*
                        let _ = terminal.draw(|f| self.draw(f));
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                        */
                    }
                }
    }
}


impl Handler<WindowRight> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: WindowRight, _ctx: &mut Context<Self>) {
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_right)
                {
                    if let Some(_target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        unimplemented!();
                        /*
                        let _ = terminal.draw(|f| self.draw(f));
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                        */
                    }
                }
    }
}

impl Handler<Quit> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: Quit, _ctx: &mut Context<Self>) {
                self.should_quit = true;
    }
}

impl Handler<AppendCommand> for Ui {
    type Result = ();

    fn handle(&mut self, msg: AppendCommand, _ctx: &mut Context<Self>) {
                if let Some(window) = self.windows.get_mut(&msg.id) {
                    window.command_text = msg.command.clone();
                }
    }
}

impl Handler<CacheNewLine> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: CacheNewLine, _ctx: &mut Context<Self>) {
                unimplemented!();
                //self.cache_new_line(&text, msg.id, msg.line_index).await;
                //self.cache_line_numbers(&text, msg.id).await;
    }
}

impl Handler<RemoveCacheLine> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: RemoveCacheLine, _ctx: &mut Context<Self>) {
                unimplemented!();
                //self.remove_cache_line(id, line_index).await;
                //self.cache_line_numbers(&text, id).await;
    }
}


impl Handler<CacheCurrentLine> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: CacheCurrentLine, _ctx: &mut Context<Self>) {
                unimplemented!();
                //self.cache_current_line(&text, msg.id, msg.line_index).await;
    }
}

impl Handler<CloseWindow> for Ui {
    type Result = ();

    fn handle(&mut self, msg: CloseWindow, _ctx: &mut Context<Self>) {
                let id = msg.id;
                let current_windows = self.windows.clone();
                let current_window = current_windows.get(&id);
                if let Some(current_window) = current_window {
                    self.windows.remove(&id);
                    unimplemented!();
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
    }
}


impl Handler<DrawViewPort> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: DrawViewPort, _ctx: &mut Context<Self>) {
                unimplemented!();
                //let _ = terminal.draw(|f| self.draw(f));
    }
}


impl Handler<SetTextLayout> for Ui {
    type Result = ();

    fn handle(&mut self, msg: SetTextLayout, _ctx: &mut Context<Self>) {
                self.add_text_split(msg.direction);
    }
}



impl Handler<SetHighlight> for Ui {
    type Result = ();

    fn handle(&mut self, _msg: SetHighlight, _ctx: &mut Context<Self>) {
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
}


impl Handler<NewWindow> for Ui {
    type Result = ();

    fn handle(&mut self, msg: NewWindow, _ctx: &mut Context<Self>) {
                let change = msg.change;
                let direction = msg.direction;
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
    }
}

impl Handler<UpdateWindow> for Ui {
    type Result = ();

    fn handle(&mut self, msg: UpdateWindow, _ctx: &mut Context<Self>) {
                let change = msg.change;
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
    }
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
    }

    pub fn remove_text_split(&mut self, window_id: Uuid) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.area = window.outer_areas.pop().flatten();
            window.bottom = window.area.map(|a| a.bottom());
            window.right = window.area.map(|a| a.right());
        }
    }

    pub fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        Self::draw_header(
            self,
            self.windows.get(&self.current_window_id),
            f,
            self.head_area,
        );
        for window in self.windows.values() {
            /*
            Self::draw_text(f, &self.highlight_cache, &self.line_num_cache, window);

            if window.id == self.current_window_id {
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
            */
            unimplemented!()
        }
        Self::draw_footer(
            self,
            self.windows.get(&self.current_window_id),
            f,
            self.foot_area,
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

    fn draw_text<B>(
        f: &mut Frame<B>,
        highlight_cache: &HashMap<Uuid, Vec<Spans>>,
        line_numbers: &HashMap<Uuid, Spans>,
        window: &Window,
    ) where
        B: Backend,
    {
        if let Some(area) = window.area {
            let inner_text_splits = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
                .split(area);
            let line_number_area = inner_text_splits[0];
            let text_area = inner_text_splits[1];
            if let Some(cached_highlights) = highlight_cache.get(&window.buffer_id) {
                let paragraph = Paragraph::new(cached_highlights.clone())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
                    .scroll((window.current_page, window.x_pos));
                f.render_widget(paragraph, text_area);
            }

            if let Some(line_numbers_cached) = line_numbers.get(&window.buffer_id) {
                let line_number_p = Paragraph::new(line_numbers_cached.clone())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
                    .scroll((window.current_page, window.x_pos));
                f.render_widget(line_number_p, line_number_area);
            }
        }
    }

    fn draw_footer<B>(&self, window: Option<&Window>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph = Paragraph::new(
            window
                .and_then(|w| w.command_text.clone())
                .unwrap_or("".to_string()),
        )
        .block(block.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
        let paragraph2 = Paragraph::new(format!(
            "{:?}",
            window.map(|w| w.mode.clone()).unwrap_or_default()
        ))
        .block(block.clone())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
        let paragraph3 = Paragraph::new(format!(
            "{},{}",
            window.map(|w| w.y_pos).unwrap_or_default(),
            window.map(|w| w.x_pos).unwrap_or_default()
        ))
        .block(block)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        f.render_widget(paragraph2, area);
        f.render_widget(paragraph3, area);
    }

    fn draw_header<B>(&self, window: Option<&Window>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph =
            Paragraph::new(window.map(|w| w.title.clone()).unwrap_or_default()).block(block);
        f.render_widget(paragraph, area);
    }

    pub fn new(terminal: &mut Term) -> AnyHowResult<Self> {
        let (head_area, text_area, foot_area) = Ui::create_layout(&terminal.get_frame());
        let ui = Self {
            should_quit: false,
            windows: HashMap::new(),
            syntax_set: None,
            theme_set: None,
            theme: None,
            syntax: None,
            current_window_id: Uuid::new_v4(),
            //highlight_cache: HashMap::new(),
            //line_num_cache: HashMap::new(),
            head_area,
            text_area,
            foot_area,
        };
        Ok(ui)
    }

    pub async fn receive_tokens(rx: Receiver<Token>, tx: Sender<Token>) -> AnyHowResult<()> {
        let stdout = stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut ui = Self::new(&mut terminal)?;
        while !ui.should_quit {
            if let Ok(token) = rx.recv_async().await {
                /*
                if let Ok(tokens) = ui.handle_token(&mut terminal, token).await {
                    for token in tokens {
                        let _ = tx.send_async(token.clone()).await;
                    }
                } else {
                    ()
                }
                */
            }
        }
        Ok(())
    }
}
