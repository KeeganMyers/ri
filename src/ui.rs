use crate::{Mode};
use crate::Window;
use std::io::Stdout;

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
    pub text_area: Rect,
    pub foot_area: Rect,
}

impl Ui {
    pub fn draw_view_port(
        &mut self,
        current_window_id: &Uuid,
        title: &Option<String>,
        mode: &Mode,
        coords: Option<(u16,u16)>,
        command_text: &Option<String>,
        window_widgets: Vec<&Window>,
        terminal: &mut Term,
    ) {
        let foot_area = self.foot_area.clone();
        let text_area = self.text_area.clone();
        let _current_window_id = self.current_window_id.clone();
        let _ = terminal.draw(|f| {
            Self::draw(
                current_window_id,
                title,
                mode,
                coords,
                command_text,
                foot_area,
                text_area,
                window_widgets,
                f,
            )
        });
    }

    pub fn split_ui(&self, window: &Window,direction: Direction) -> Vec<Rect> {
        let text_area = if let Some(area) = window.area
        {
            area
        } else {
            self.text_area
        };
        Layout::default()
            .direction(direction)
            .constraints(
                vec![Constraint::Percentage(50),Constraint::Percentage(50)]
            )
            .split(text_area)
    }

    pub fn draw<'a, B: 'a>(
        current_window_id: &Uuid,
        title: &Option<String>,
        mode: &Mode,
        coords: Option<(u16,u16)>,
        command_text: &Option<String>,
        foot_area: Rect,
        text_area: Rect,
        window_widgets: Vec<&Window>,
        f: &mut Frame<'a, B>,
    ) where
        B: Backend,
    {
        for window in window_widgets {
            if window.id == *current_window_id {
                f.set_cursor(window.cursor_x_pos(), window.cursor_y_pos());
            }

            f.render_widget(window, text_area);
        }

        Self::draw_footer(mode,coords, command_text,f, foot_area);
    }

    fn create_layout<B: Backend>(frame: &Frame<B>) -> (Rect, Rect) {
        let area = Layout::default()
            .constraints(
                [
                    Constraint::Min(20),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(frame.size());
        (area[0], area[1])
    }

    fn draw_footer<B>(mode: &Mode, coords: Option<(u16,u16)>,command_text: &Option<String>, f: &mut Frame<B>, area: Rect)
    where
        B: Backend,
    {
        let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
        let paragraph = Paragraph::new(command_text.clone().unwrap_or_default())
        .block(block.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
        let paragraph2 = Paragraph::new(format!("{:?}",mode))
            .block(block.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        let paragraph3 = Paragraph::new(format!(
            "{},{}",
            coords.unwrap_or_default().0,
            coords.unwrap_or_default().1
        ))
        .block(block)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        f.render_widget(paragraph2, area);
        f.render_widget(paragraph3, area);
    }


    pub fn new(terminal: &mut Term) -> Self {
        let (text_area, foot_area) = Ui::create_layout(&terminal.get_frame());
        Self {
            should_quit: false,
            current_window_id: Uuid::new_v4(),
            text_area,
            foot_area,
            ..Self::default()
        }
    }
}
