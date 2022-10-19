use crate::{App, Window};
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn control_style() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Gray)
        .add_modifier(Modifier::BOLD)
}

pub fn convert_style(style: SyntectStyle) -> Style {
    Style::default()
        .fg(Color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b,
        ))
        /*
        .bg(Color::Rgb(
            style.background.r,
            style.background.g,
            style.background.b,
        ))
        */
}

pub fn to_span(style: SyntectStyle, value: &str) -> Span {
    Span::styled(value, convert_style(style))
}

pub fn to_spans(highlights: Vec<(SyntectStyle, &str)>) -> Spans {
    Spans::from(
        highlights
            .iter()
            .map(|h| to_span(h.0, h.1))
            .collect::<Vec<Span>>(),
    )
}


pub fn draw<B: Backend>(f: &mut Frame<B>, app: & mut App) {
    let area = Layout::default()
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Percentage(95),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.size());
    let text_area = area[1];
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
    app.set_y_offset(text_area.top());
    app.set_x_offset(4);
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
    let inner_text_splits = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(4),
                Constraint::Percentage(95),
            ]
            .as_ref(),
        )
        .split(*area);
    let line_number_area = inner_text_splits[0];
    let text_area = inner_text_splits[1];
    let y_cursor = if app.display_y_pos() >= area.bottom() {
        area.bottom() - 3
    } else {
        app.display_y_pos()
    };

    let x_cursor = if app.display_x_pos() >= area.right() {
        area.right() - 1
    } else {
        app.display_x_pos()
    };
        if let Some(formatted_text) = app.window_at(window.buffer_idx as usize).and_then(|w| w.formatted_text) {
            let paragraph = if let Some(current_page) = app.current_page() {
                Paragraph::new(formatted_text)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
                    .scroll((current_page, app.x_pos()))
            } else {
                Paragraph::new(formatted_text)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
            };
            f.render_widget(paragraph, text_area);
            f.set_cursor(x_cursor, y_cursor);
        }
        if let Some(line_numbers) = app.window_at(window.buffer_idx as usize).and_then(|w| w.line_numbers) {
            let line_number_p = if let Some(current_page) = app.current_page() {
                Paragraph::new(line_numbers)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
                    .scroll((current_page, app.x_pos()))
            } else {
                Paragraph::new(line_numbers)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false })
            };
            f.render_widget(line_number_p, line_number_area);
        }
}

fn draw_footer<B>(f: &mut Frame<B>, app: & mut App, area: Rect)
where
B: Backend,
{
    let block = Block::default().style(control_style());
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

fn draw_header<B>(f: &mut Frame<B>, app: & mut App, area: Rect)
where
    B: Backend,
{
    let block = Block::default().style(
        control_style()
    );
    let text = format!("{}", app.title());
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
