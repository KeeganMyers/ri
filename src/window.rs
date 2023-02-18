use crate::{Buffer,app::Mode, ui::Term,token::{display_token::*, command_token::*,normal_token::*, Token}};
use uuid::Uuid;
use actix::prelude::*;
use tui::{widgets::Widget,
    text::Spans,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    buffer::Buffer as TuiBuffer
};

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

#[derive(Default, Clone, Debug)]
pub struct Window {
//pub struct Window<'a> {
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
    pub terminal: Arc<Term>,
}

impl Widget for Window {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        if let Some(area) = self.area {
            let inner_text_splits = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
                .split(area);
            let line_number_area = inner_text_splits[0];
            let text_area = inner_text_splits[1];
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
    pub fn new(buffer: &Buffer) -> Self {
            Self {
                terminal: Arc<Terminal>,
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
        match token {
        /*
         DisplayToken::DrawWindow(terminal,area) => {
            self.area = Some(area);
            let _ = terminal.draw(|f| f.render_widget(self.clone(), f.size()));
         },
        */
         _ => ()
        };
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
                    ()
                },
                _ => ()
        }
    }
}

