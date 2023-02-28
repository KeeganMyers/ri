use crate::{Buffer,app::Mode, ui::Term,token::{display_token::*, command_token::*,normal_token::*, Token,GetState}};
use uuid::Uuid;
use actix::prelude::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use syntect::util::LinesWithEndings;
use std::sync::{Mutex,Arc};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Wrap},
    Frame, Terminal,
    buffer::Buffer as TuiBuffer,
    widgets::Widget,
};
use compressed_string::ComprString;
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

#[derive(Clone)]
pub struct CachedSpan {
    pub content: ComprString,
    pub style: Style
}

impl<'a> From<Span<'a>> for CachedSpan {
    fn from(span: Span) -> Self {
        CachedSpan {
            content: ComprString::new(span.content.deref()),
            style: span.style
        }
    }
}

#[derive(Default, Clone, MessageResponse)]
pub struct Window {
    pub id: Uuid,
    pub title: String,
    pub current_percent_size: u16,
    pub buffer_id: Uuid,
    pub y_offset: u16,
    pub x_offset: u16,
    pub x_pos: u16,
    pub y_pos: u16,
    pub mode: Mode,
    pub page_size: u16,
    pub current_page: u16,
    pub outer_areas: Vec<Option<Rect>>,
    pub area: Option<Rect>,
    pub command_text: Option<String>,
    pub bottom: Option<u16>,
    pub right: Option<u16>,
    pub window_left: Option<Uuid>,
    pub window_right: Option<Uuid>,
    pub window_up: Option<Uuid>,
    pub window_down: Option<Uuid>,
    pub highlight_cache: Vec<Vec<CachedSpan>>,
    pub line_num_cache: Vec<Vec<CachedSpan>>,
}

impl Widget for Window {
    fn render(self, _area: Rect, buf: &mut TuiBuffer) {
        if let Some(area) = self.area {
            let inner_text_splits = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
                .split(area);
            //let line_number_area = inner_text_splits[0];
            let text_area = inner_text_splits[1];

            let spans = Spans::from(vec![Span::raw("hello world")]);
                /*
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .scroll((self.current_page, self.x_pos));
                term_lock.get_frame().render_widget(paragraph, text_area);
                */

            buf.set_spans(text_area.x, text_area.y, &spans, text_area.width);

            /*
            if let Some(cached_highlights) = highlight_cache.get(&window.buffer_id) {
                let paragraph = Paragraph::new(cached_highlights.clone())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
                    .scroll((window.current_page, window.x_pos));
                f.render_widget(paragraph, text_area);
            }
            */
            /*
            if let Some(line_numbers_cached) = line_numbers.get(&window.buffer_id) {
                let line_number_p = Paragraph::new(line_numbers_cached.clone())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
                    .scroll((window.current_page, window.x_pos));
                f.render_widget(line_number_p, line_number_area);
            }
            */
        }
    }
}

impl Window {
    pub fn convert_style(style: SyntectStyle) -> Style {
        Style::default().fg(Color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b,
        ))
    }

    pub fn to_span(style: SyntectStyle, value: &str) -> CachedSpan {
        CachedSpan {
            content: ComprString::new(value),
            style: Self::convert_style(style)
        }
    }

    pub fn to_spans(highlights: Vec<(SyntectStyle, &str)>) -> Spans {
        todo!("convert from vec to spans");
        /*
        Spans::from(
            highlights
                .iter()
                .map(|h| Self::to_span(h.0, h.1))
                .collect::<Vec<Span>>(),
        )
        */
    }

    pub fn draw<'a,B: 'a> (main_area: Option<Rect>,f: &mut Frame<'a,B>) 
        where 
            B: Backend
    {
        if let Some(area) = main_area {
            let inner_text_splits = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
                .split(area);
            let line_number_area = inner_text_splits[0];
            let text_area = inner_text_splits[1];
            let paragraph = Paragraph::new(Spans::from(vec![Span::raw("hello world")]))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });
                //.scroll((self.current_page, self.x_pos));
                //term_lock.get_frame().render_widget(paragraph, text_area);
                f.render_widget(paragraph, text_area);
        }
    }

    pub fn new(buffer: &Buffer) -> Self {
            Self {
                id: Uuid::new_v4(),
                buffer_id: buffer.id,
                x_pos: buffer.x_pos,
                y_pos: buffer.y_pos,
                mode: buffer.mode.clone(),
                title: buffer.title.clone(),
                page_size: buffer.page_size,
                current_page: buffer.current_page,
                y_offset: 0,
                x_offset: 4,
                ..Window::default()
            }
            /*
            let _ = tx
                .send_async(Token::Display(DisplayToken::CacheWindowContent(
                    buffer.id,
                    buffer.text.clone(),
                )))
                .await;
            */
    }

    pub fn display_x_pos(&self) -> u16 {
        self.x_pos + self.x_offset
    }

    pub fn display_y_pos(&self) -> u16 {
        (self.y_pos + self.y_offset) - self.current_page
    }

    pub fn get_origin(&self) -> Option<(u16, u16)> {
        if let Some(area) = self.area {
            return Some((area.x, area.y));
        }
        None
    }

    fn handle_display_token(
        &mut self,
        token: DisplayToken,
    )  {
        /*
        match token {
         DisplayToken::DrawWindow => {
                let main_area = self.area;
                if let Some(mut term) = self.terminal.as_ref().and_then(|t| t.lock().ok()) {
                    term.draw(|f| Self::draw(main_area,f));
                }
         },
         _ => ()
        };
        */
        ()
    }
}

impl Actor for Window {
    type Context = Context<Self>;
}

impl Handler<Token> for Window {
        type Result = ();

        fn handle(&mut self, msg: Token , ctx: &mut Context<Self>) -> Self::Result {
            match msg {
                Token::Display(t) => {
                    self.handle_display_token(t);
                },
                _ => ()
        }
    }
}

impl Handler<GetState> for Window {
        type Result = Window;

        fn handle(&mut self, msg: GetState , ctx: &mut Context<Self>) -> Self::Result {
            self.clone()
        }
}

