use crate::{App, Window};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Span, Spans},
    widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle},
    widgets::{
        Axis, BarChart, Block, Borders, Cell, Chart, Dataset, Gauge, LineGauge, List, ListItem,
        Paragraph, Row, Sparkline, Table, Tabs, Wrap,
    },
    Frame,
};

//pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let area = Layout::default()
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());
    let text_area = area[1];
    //nest a layout
    let text_splits = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            app.windows
                .iter()
                .map(|w| Constraint::Percentage(w.current_percent_size))
                .collect::<Vec<Constraint>>()
                .as_ref(),
        )
        .split(text_area);
    app.set_y_offset(area[0].bottom());
    draw_header(f, app, area[0]);
    for (split, window) in text_splits.iter().zip(app.windows.iter()) {
        draw_text(f, app, split, window)
    }
    draw_footer(f, app, area[2]);
}

fn draw_text<B>(f: &mut Frame<B>, app: &App, area: &Rect, window: &Window)
where
    B: Backend,
{
    if let Some(text) = app.buffer_at(window.buffer_idx as usize) {
        let block = Block::default().borders(Borders::ALL);
        let paragraph = Paragraph::new(text.to_string())
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, *area);
    }
}

fn draw_footer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL);
    //let text = app.command_text.clone().unwrap_or("test".to_string());
    let paragraph = Paragraph::new(app.command_text().unwrap_or("".to_string()))
        .block(block.clone())
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    let paragraph2 = Paragraph::new(format!("{:?}", app.mode()))
        .block(block.clone())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    let paragraph3 = Paragraph::new(format!("{},{}", app.y_pos(), app.x_pos()))
        .block(block)
        .alignment(Alignment::Right)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
    f.render_widget(paragraph2, area);
    f.render_widget(paragraph3, area);
}

fn draw_header<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        "Header",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let text = format!("{},{}", app.y_pos(), app.x_pos());
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
