use crate::{App, Window};
use std::collections::HashMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use syntect::util::LinesWithEndings;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use uuid::Uuid;

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

pub fn line_number_spans(line_number_count: usize) -> Spans<'static> {
    Spans::from(
        (1..line_number_count)
            .map(|l| Span::styled(format!("{:<5}", l), Style::default().fg(Color::Yellow)))
            .collect::<Vec<Span>>(),
    )
}

pub fn draw<B: Backend>(
    f: &mut Frame<B>,
    app: &mut App,
    highlight_cache: &mut HashMap<Uuid, Vec<Spans>>,
    line_numbers: &mut HashMap<Uuid, Spans>,
) {
    let area = Layout::default()
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(20),
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
        draw_text(f, app, highlight_cache, line_numbers, split, window)
    }

    draw_footer(f, app, area[2]);
}

fn draw_text<B>(
    f: &mut Frame<B>,
    app: &App,
    highlight_cache: &mut HashMap<Uuid, Vec<Spans>>,
    line_numbers: &mut HashMap<Uuid, Spans>,
    area: &Rect,
    window: &Window,
) where
    B: Backend,
{
    let mut highlight =
        HighlightLines::new(&app.syntax, &app.theme_set.themes["base16-ocean.dark"]);
    let mut spans: Vec<Spans> = vec![];

    let inner_text_splits = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
        .split(*area);
    let line_number_area = inner_text_splits[0];
    let text_area = inner_text_splits[1];
    if let (Some(text), Some(id)) = (
        app.buffer_at(window.buffer_idx as usize),
        app.buffer_id_at(window.buffer_idx as usize),
    ) {
        if let Some(cached_highlights) = highlight_cache.get(&id) {
            spans = cached_highlights.clone();
        } else {
            let rope_str = text.to_string();
            let text_lines = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            for line in text_lines {
                if let Ok(hs) = highlight.highlight_line(line, &app.syntax_set) {
                    spans.push(to_spans(hs.clone()));
                }
            }
            highlight_cache.insert(id, spans.clone());
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
        let line_number_spans = if let Some(line_numbers_cached) = line_numbers.get(&id) {
            line_numbers_cached.clone()
        } else {
            let line_count = text.len_lines();
            let local_line_nums = line_number_spans(line_count);
            line_numbers.insert(id, local_line_nums.clone());
            local_line_nums
        };

        let line_number_p = if let Some(current_page) = app.current_page() {
            Paragraph::new(line_number_spans)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false })
                .scroll((current_page, app.x_pos()))
        } else {
            Paragraph::new(line_number_spans)
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
    let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
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
    let block = Block::default().style(Style::default().fg(Color::Black).bg(Color::White));
    let paragraph = Paragraph::new(app.title().unwrap_or_default()).block(block);
    f.render_widget(paragraph, area);
}
