use syntect::easy::HighlightLines;
use ropey::Rope;
use syntect::highlighting::Style as SyntectStyle;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use syntect::{
    highlighting::ThemeSet,
    parsing::{SyntaxReference, SyntaxSet},
};
use crate::{
    util::event::{Config, Event},
    Command, NormalCommand, OperatorCommand, RangeCommand,
};

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

pub struct Window<'a> {
    pub title: String,
    pub should_quit: bool,
    pub current_percent_size: u16,
    pub buffer_idx: u16,
    pub command_text: Option<String>,
    pub operator: Option<OperatorCommand>,
    pub y_offset: u16,
    pub x_offset: u16,
    pub x_pos: u16,
    pub y_pos: u16,
    pub page_size: u16,
    pub current_page: u16,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub syntax: SyntaxReference,
    pub formatted_text: Option<Vec<Spans<'a>>>,
    pub line_numbers: Option<Spans<'a>>,
    pub highlight: HighlightLines<'a>,
}

impl<'a> Window<'a> {
    pub fn new(buffer_idx: u16,buffer: Option<Rope>) -> Self {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = ps.find_syntax_by_extension("rs").expect("could not load syntax");
        let theme = ts.themes["base16-eighties.dark"].clone();
        let mut highlight = HighlightLines::new(syntax,&theme);
        let formatted_text = Self::format_text(buffer,&ps,&mut highlight);
        let line_numbers = Self::line_number_spans(&buffer);
        Self {
            title: "".to_string(),
            should_quit: false,
            highlight,
            current_percent_size: 50,
            buffer_idx,
            syntax_set: ps.clone(),
            theme_set: ts,
            syntax: syntax.to_owned(),
            formatted_text,
            line_numbers,
            x_pos: 0,
            y_pos: 0,
            x_offset: 0,
            y_offset: 0,
            command_text: None,
            operator: None,
            current_page: 0,
            page_size: 10,
        }
    }

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

    pub fn format_text<'f>(buffer: Option<Rope>,ps: &SyntaxSet,highlight: &mut HighlightLines) -> Option<Vec<Spans<'f>>> {
        if let Some(text) = buffer {
            let mut spans: Vec<Spans> = vec![];
            for line in text.lines() {
                if let Some(l) = line.as_str() {
                    if let Ok(highlights) = highlight.highlight_line(l, ps) {
                        spans.push(Self::to_spans(highlights));
                    }
                }
            }
            return Some(spans);
        }
        None
    }

    pub fn line_number_spans<'f>(buffer: &Option<Rope>) -> Option<Spans<'f>> {
        if let Some(text) = buffer {
            let line_count = text.len_lines();
            return Some(Spans::from(
            (1..line_count).map(|l|  Span::styled(format!("{:<5}",l), Style::default()

                .fg(Color::Yellow)))
                    .collect::<Vec<Span>>(),
            ));
        }
        None
    }
}
