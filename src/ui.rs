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

pub fn convert_style(style: SyntectStyle) -> Style {
    Style::default()
        .fg(Color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b,
        ))
        .bg(Color::Rgb(
            style.background.r,
            style.background.g,
            style.background.b,
        ))
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

pub fn line_number_spans(line_number_count: &usize) -> Spans {
    Spans::from(
    (1..*line_number_count).map(|l|  Span::styled(format!("{:<5}",l), Style::default()

        .fg(Color::Yellow)))
            .collect::<Vec<Span>>(),
    )
}

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
    app.set_y_offset(text_area.top() + 1);
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
    let mut highlight = HighlightLines::new(&app.syntax, &app.theme_set.themes["base16-ocean.dark"]);
    let block = Block::default().borders(Borders::ALL);
    let mut spans: Vec<Spans> = vec![];

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
    if let Some(text) = app.buffer_at(window.buffer_idx as usize) {
        for line in text.lines() {
            if let Some(l) = line.as_str() {
                if let Ok(highlights) = highlight.highlight_line(l, &app.syntax_set) {
                    spans.push(to_spans(highlights));
                }
            }
        }
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
        //text
        let paragraph = if let Some(current_page) = app.current_page() {
            Paragraph::new(spans)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .scroll((current_page, app.x_pos()))
        } else {
            Paragraph::new(spans)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
        };
        //line numbers
        let line_count = text.len_lines();
        let line_numbers = line_number_spans(&line_count);
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
        f.render_widget(paragraph, text_area);
        f.set_cursor(x_cursor, y_cursor);
    }
}

fn draw_footer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL);
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
