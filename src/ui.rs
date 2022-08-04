use crate::App;
use tui::{
    backend::Backend,
    layout::{Alignment,Constraint, Direction, Layout, Rect},
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
        .constraints([Constraint::Percentage(10), 
                      Constraint::Percentage(85),
                      Constraint::Percentage(5),
                     ].as_ref())
        .split(f.size());

    draw_header(f, app,area[0]);
    draw_first_tab(f, app, area[1]);
    draw_footer(f, app,area[2]);
}


fn draw_first_tab<B>(f: &mut Frame<B>, app: &mut App, chunks: Rect)
where
    B: Backend,
{
    draw_text(f, app, chunks);
}

fn draw_text<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let paragraph = Paragraph::new(format!("{}",app.text));
    f.render_widget(paragraph, area);
}


fn draw_footer<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let block = Block::default().title(Span::styled(
        format!("{},{}",app.y_pos, app.x_pos),
        Style::default()))
        .title_alignment(Alignment::Right);
    let text = app.command_text.clone().unwrap_or("".to_string());
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
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
    let text = format!("{},{}",app.y_pos, app.x_pos);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}
