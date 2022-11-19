use crate::Window;
use ropey::Rope;
use flume::{Sender,Receiver};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crate::token::{
    AppendToken, CommandToken, InsertToken, NormalToken, OperatorToken, RangeToken, Token,DisplayToken
};
use std::io::{stdout,Stdout};
use termion::{input::MouseTerminal, raw::{RawTerminal,IntoRawMode}, screen::AlternateScreen};
use std::collections::HashMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use syntect::util::LinesWithEndings;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};

use tui::{
    backend::{TermionBackend, Backend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Wrap},
    Frame,
    Terminal
};
use uuid::Uuid;
pub type Term = Terminal<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<Stdout>>>>>;
pub struct Ui<'a> {
    pub windows: HashMap<Uuid, Window>,
    pub syntax_set: Option<SyntaxSet>,
    pub theme_set: Option<ThemeSet>,
    pub theme: Option<Theme>,
    pub syntax: Option<SyntaxReference>,
    pub command_text: Option<String>,
    pub current_window_id: Uuid,
    pub head_area: Rect,
    pub text_area: Rect,
    pub foot_area: Rect,
    pub highlight_cache: HashMap<Uuid, Vec<Spans<'a>>>,
    pub line_num_cache: HashMap<Uuid, Spans<'a>>,
}

impl<'a> Ui<'a> {
    pub fn convert_style(style: SyntectStyle) -> Style {
        Style::default()
            .fg(Color::Rgb(
                style.foreground.r,
                style.foreground.g,
                style.foreground.b,
            ))
            .bg(Color::Black)
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

    pub fn draw<B: Backend>(
        &self,
        f: &mut Frame<B>
    ) {
        Self::draw_header(
            self,
            self.windows.get(&self.current_window_id),
            f,
            self.head_area,
        );
        let text_splits = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                self.windows
                    .values()
                    .map(|w| Constraint::Percentage(w.current_percent_size))
                    .collect::<Vec<Constraint>>()
                    .as_ref(),
            )
            .split(self.text_area);
        for (split, window) in text_splits.iter().zip(self.windows.values()) {
            //window.y_offset = self.text_area.top();
            //window.x_offset = 4;
            Self::draw_text(f, &self.highlight_cache, &self.line_num_cache, split, window)
        }
        Self::draw_footer(
            self,
            self.windows.get(&self.current_window_id),
            f,
            self.foot_area,
        );
    }

    fn create_layout<B: Backend>(frame: &Frame<B>) -> (Rect,Rect,Rect) {
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
        (area[0],area[1],area[2])
    }

    fn cache_formatted_text(&self,text: Rope,window: Window,highlight_cache: &mut HashMap<Uuid, Vec<Spans>>) {
        //&self.theme_set.themes["base16-ocean.dark"];
        let theme = self.theme.clone().unwrap();
        let mut highlight = HighlightLines::new(&self.syntax.clone().unwrap(), &theme);
        let mut spans: Vec<Spans> = vec![];
        let rope_str = text.to_string();
        let text_lines = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
        for line in text_lines {

            if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                spans.push(Self::to_spans(hs.clone()));
            }
        }
        highlight_cache.insert(window.buffer_id, spans.clone());
    }

    fn cache_line_numbers(&self,text: Rope,window: Window,line_numbers: &mut HashMap<Uuid, Spans>) {
                let line_count = text.len_lines();
                let local_line_nums = Self::line_number_spans(line_count);
                line_numbers.insert(window.buffer_id, local_line_nums.clone());
    }

    fn draw_text<B>(
        f: &mut Frame<B>,
        highlight_cache: &HashMap<Uuid, Vec<Spans>>,
        line_numbers: &HashMap<Uuid, Spans>,
        area: &Rect,
        window: &Window,
    ) where
        B: Backend,
    {
        let inner_text_splits = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
            .split(*area);
        let line_number_area = inner_text_splits[0];
        let text_area = inner_text_splits[1];

        if let Some(cached_highlights) = highlight_cache.get(&window.buffer_id) {
            let paragraph = Paragraph::new(cached_highlights.clone())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .scroll((window.current_page, window.x_pos));
            f.render_widget(paragraph, text_area);
        }

        let y_cursor = if window.display_y_pos() >= area.bottom() {
            area.bottom() - 3
        } else {
            window.display_y_pos()
        };

        let x_cursor = if window.display_x_pos() >= area.right() {
            area.right() - 1
        } else {
            window.display_x_pos()
        };

        if let Some(line_numbers_cached) = line_numbers.get(&window.buffer_id) {
            let line_number_p = Paragraph::new(line_numbers_cached.clone())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .scroll((window.current_page, window.x_pos));
            f.render_widget(line_number_p, line_number_area);
        }

        f.set_cursor(x_cursor, y_cursor);
    }

    fn draw_footer<B>(&self, window: Option<&Window>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph = Paragraph::new(self.command_text.clone().unwrap_or("".to_string()))
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

    fn handle_display_token(&mut self,terminal: &mut Term,token: DisplayToken) -> AnyHowResult<Vec<Token>> {
        match token {
           DisplayToken::DrawViewPort => {
            let _ = terminal.draw(|f| {
                self.draw(f) 
            });
            ()
           },
            _ => (),
        };
        
        Ok(vec![])
    }

    pub fn handle_token(&mut self, terminal: &mut Term,token: Token) -> AnyHowResult<Vec<Token>> {
        let _ = match token {
            Token::Display(t) => self.handle_display_token(terminal,t),
            /*
            Token::Append(t) => self.handle_append_token(t),
            Token::Command(t) => self.handle_command_token(t),
            Token::Insert(t) => self.handle_insert_token(t),
            Token::Normal(t) => self.handle_normal_token(t),
            Token::Operator(t) => self.handle_operator_token(t),
            Token::Range(t) => self.handle_range_token(t),
            */
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        };
        Ok(vec![])
    }

    pub fn new(terminal: &mut Term) -> AnyHowResult<Self> {
        let (head_area,text_area,foot_area) = Ui::create_layout(&terminal.get_frame());
        let ui = Self {
            windows: HashMap::new(),
            syntax_set: None,
            theme_set: None,
            theme: None,
            syntax: None,
            command_text: None,
            current_window_id: Uuid::new_v4(),
            highlight_cache: HashMap::new(),
            line_num_cache: HashMap::new(),
            head_area,
            text_area,
            foot_area
        };
       Ok(ui) 
    }

    pub async fn receive_tokens(
        rx: Receiver<Token>,
        tx: Sender<Token>) -> AnyHowResult<()>
    {
        let stdout = stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut ui = Self::new(&mut terminal)?;
        log::info!("starting display");
        while let Ok(token) = rx.recv_async().await {
            log::info!("display got token {:?}",token);
            let _ = ui.handle_token(&mut terminal,token);
        }
        log::info!("got no tokens closing display");
        Ok(())
    }
}


