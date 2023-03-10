use crate::token::{display_token::*, Token};
//use crate::{token::{display_token::*, command_token::*,normal_token::*, Token}};
use crate::Window;
use actix::prelude::*;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use std::io::Stdout;
use std::sync::{Arc, Mutex};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph, Wrap},
    Frame, Terminal,
};
use uuid::Uuid;
pub type Term = Terminal<CrosstermBackend<Stdout>>;

#[derive(Default)]
pub struct Ui {
    pub should_quit: bool,
    pub current_window_id: Uuid,
    pub head_area: Rect,
    pub text_area: Rect,
    pub foot_area: Rect,
}

impl Actor for Ui {
    type Context = Context<Self>;
}

impl Ui {
    pub fn draw_view_port(&mut self, current_window_id: &Uuid, window_widgets:  Vec<&Window>,terminal: &mut Term) {
                let head_area = self.head_area.clone();
                let foot_area = self.foot_area.clone();
                let text_area = self.text_area.clone();
                let _current_window_id = self.current_window_id.clone();
                let _ = terminal.draw(|f| {
                    Self::draw(
                        current_window_id,
                        head_area,
                        foot_area,
                        text_area,
                        window_widgets,
                        f,
                    )
                });
    }

    pub fn add_text_split(&mut self, _direction: Direction) {
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

    /*
    pub fn remove_text_split(&mut self, _window_id: Uuid) {
        if let Some(window) = self.windows.get_mut(&window_id) {
            window.area = window.outer_areas.pop().flatten();
            window.bottom = window.area.map(|a| a.bottom());
            window.right = window.area.map(|a| a.right());
        }
    }
        */

    pub fn draw<'a, B: 'a>(
        current_window_id: &Uuid,
        head_area: Rect,
        foot_area: Rect,
        text_area: Rect,
        window_widgets: Vec<&Window>,
        f: &mut Frame<'a, B>,
    ) where
        B: Backend,
    {
        Self::draw_header(None, f, head_area);

        for window in window_widgets {
            if window.id == *current_window_id {
                f.set_cursor(window.cursor_x_pos(), window.cursor_y_pos());
            }

            f.render_widget(window, text_area);
        }

        Self::draw_footer(None, f, foot_area);
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

    fn draw_footer<B>(window: Option<Window>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph = Paragraph::new(
            window
                .clone()
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
            Paragraph::new(window.and_then(|w| w.title.clone()).unwrap_or_default()).block(block);
        f.render_widget(paragraph, area);
    }

    pub fn new(terminal: &Term) -> Self {
        let (head_area, text_area, foot_area) = Ui::create_layout(&terminal.get_frame());
        Self {
            should_quit: false,
            current_window_id: Uuid::new_v4(),
            head_area,
            text_area,
            foot_area,
            ..Self::default()
    }
    }

    fn handle_display_token(&mut self, token: DisplayToken) -> AnyHowResult<Vec<Token>> {
        match token {
            DisplayToken::NewWindow(_change, _direction) => {
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
            DisplayToken::CloseWindow(_id) => {
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
            _ => (),
        };

        Ok(vec![])
    }
    /*
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
    */

    /*
    async fn handle_normal_token(
        &mut self,
        //terminal: &mut Term,
        token: NormalToken,
    ) -> AnyHowResult<Vec<Token>> {
        match token {
            NormalToken::WindowLeft => {
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_left)
                {
                    if let Some(target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        let _ = terminal.draw(|f| self.draw(f));
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                    }
                }
                Ok(vec![])
            }
            NormalToken::WindowRight => {
                if let Some(window_id) = self
                    .windows
                    .get(&self.current_window_id)
                    .and_then(|w| w.window_right)
                {
                    if let Some(target_window) = self.windows.get(&window_id) {
                        self.current_window_id = window_id;
                        let _ = terminal.draw(|f| self.draw(f));
                        return Ok(vec![Token::Command(CommandToken::SetBuffer(
                            target_window.buffer_id,
                        ))]);
                    }
                }
                Ok(vec![])
            }
            NormalToken::WindowUp => {
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
                Ok(vec![])
            }
            NormalToken::WindowDown => {
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
                Ok(vec![])
            }
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        }
    }
    */
}

impl Handler<Token> for Ui {
    type Result = ();

    fn handle(&mut self, msg: Token, _ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            Token::Display(t) => {
                let _ = self.handle_display_token(t);
                ()
            }
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
