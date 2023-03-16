use crate::{
    reflow::{LineComposer, WordWrapper},
    token::{display_token::*},
};
use ropey::Rope;
use std::iter;
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::Style as SyntectStyle;
use syntect::util::LinesWithEndings;
use syntect::{
    highlighting::{Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
};
use tui::{
    buffer::Buffer as TuiBuffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, StyledGrapheme},
    widgets::{BorderType,Borders,Widget}
};
use uuid::Uuid;

use compressed_string::ComprString;
use std::ops::Deref;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Copy, Clone)]
pub struct WindowPosition {
    pub y: u32,
    pub x: u32,
}

#[derive(Clone)]
pub struct CachedSpan {
    pub content: ComprString,
    pub style: Style,
}

impl CachedSpan {
    #[allow(dead_code)]
    fn raw(content: &str) -> Self {
        CachedSpan {
            content: ComprString::from(content),
            style: Style::default(),
        }
    }
}

impl<'a> From<&CachedSpan> for Span<'a> {
    fn from(span: &CachedSpan) -> Self {
        Span {
            content: span.content.to_string().into(),
            style: span.style,
        }
    }
}

#[derive(Default, Clone)]
pub struct Window {
    pub id: Uuid,
    pub order: usize,
    pub title: Option<String>,
    pub current_percent_size: u16,
    pub y_offset: u16,
    pub x_offset: u16,
    pub x_pos: u16,
    pub y_pos: u16,
    pub page_size: u16,
    pub current_page: u16,
    pub area: Option<Rect>,
    pub command_text: Option<String>,
    pub highlight_cache: Vec<Vec<CachedSpan>>,
    pub line_num_cache: Vec<Vec<CachedSpan>>,
    pub syntax_set: Option<SyntaxSet>,
    pub theme_set: Option<Arc<ThemeSet>>,
    pub theme: Option<Theme>,
    pub syntax: Option<SyntaxReference>,
}

impl Widget for &Window {
    fn render(self, _area: Rect, buf: &mut TuiBuffer) {
        if let Some(area) = self.area {
            self.render_border(&area,buf);
            let main_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Percentage(99)].as_ref())
                .split(area);
            let header_area = main_area[0];
            self.render_header(format!("{} {}",self.order,self.title.clone().unwrap_or_default()),&header_area,buf);
            let inner_text_splits = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(4), Constraint::Percentage(95)].as_ref())
                .split(main_area[1]);
            let line_number_area = inner_text_splits[0];
            let text_area = inner_text_splits[1];
            let spans = self
                .highlight_cache
                .iter()
                .map(|outer| {
                    Spans::from(
                        outer
                            .iter()
                            .map(|inner| inner.deref().into())
                            .collect::<Vec<Span>>(),
                    )
                })
                .collect::<Vec<Spans>>();

            let line_number_spans = self
                .line_num_cache
                .iter()
                .map(|outer| {
                    Spans::from(
                        outer
                            .iter()
                            .map(|inner| inner.deref().into())
                            .collect::<Vec<Span>>(),
                    )
                })
                .collect::<Vec<Spans>>();
            self.render_text(line_number_area, line_number_spans, buf);
            self.render_text(text_area, spans, buf);
        }
    }
}

impl Window {
    fn render_header(&self, title: String,area: &Rect,buf: &mut TuiBuffer)
    {
        let style = Style::default().fg(Color::Black).bg(Color::White);
        let mut title_chars = title[..].chars();
        for x in area.left()..area.right() {
            if let Some(current_char) = title_chars.next() {
                buf.get_mut(x, area.top())
                    .set_symbol(&current_char.to_string())
                    .set_style(style);
            } else {
                buf.get_mut(x, area.top())
                    .set_symbol(" ")
                    .set_style(style);
            }
        }
    }

    fn render_border(&self, area: &Rect,buf: &mut TuiBuffer) {
        let style = Style::default();
        buf.set_style(*area, style);
        let symbols = BorderType::line_symbols(BorderType::Rounded);

        // Sides
        // left
            for y in area.top()..area.bottom() {
                buf.get_mut(area.left(), y)
                    .set_symbol(symbols.vertical)
                    .set_style(style);
            }
            //top
            for x in area.left()..area.right() {
                buf.get_mut(x, area.top())
                    .set_symbol(symbols.horizontal)
                    .set_style(style);
            }
            //right
            let x = area.right() - 1;
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(symbols.vertical)
                    .set_style(style);
            }
            //bottom
            let y = area.bottom() - 1;
            for x in area.left()..area.right() {
                buf.get_mut(x, y)
                    .set_symbol(symbols.horizontal)
                    .set_style(style);
            }

        // Corners
        // bottom right
            buf.get_mut(area.right() - 1, area.bottom() - 1)
                .set_symbol(symbols.bottom_right)
                .set_style(style);
        //top right
            buf.get_mut(area.right() - 1, area.top())
                .set_symbol(symbols.top_right)
                .set_style(style);
        //left bottom
            buf.get_mut(area.left(), area.bottom() - 1)
                .set_symbol(symbols.bottom_left)
                .set_style(style);
        //top left
            buf.get_mut(area.left(), area.top())
                .set_symbol(symbols.top_left)
                .set_style(style);
    }

    pub fn cache_window_content(&mut self, text: &Rope) {
        self.cache_formatted_text(&text);
        self.cache_line_numbers(&text);
    }

    pub fn set_highlight(&mut self) {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = ps.find_syntax_by_extension("rs").clone();
        let theme = ts.themes["base16-ocean.dark"].clone();
        self.syntax_set = Some(ps.clone());
        self.theme_set = Some(Arc::new(ts));
        if let Some(s) = syntax {
            self.syntax = Some(s.clone());
        }
        self.theme = Some(theme);
    }

    fn convert_style(style: SyntectStyle) -> Style {
        Style::default().fg(Color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b,
        ))
    }

    fn to_cached_span(style: SyntectStyle, value: &str) -> CachedSpan {
        CachedSpan {
            content: ComprString::new(value),
            style: Self::convert_style(style),
        }
    }

    fn to_spans(highlights: Vec<(SyntectStyle, &str)>) -> Vec<CachedSpan> {
        highlights
            .iter()
            .map(|h| Self::to_cached_span(h.0, h.1))
            .collect::<Vec<CachedSpan>>()
    }

    pub fn cache_formatted_text(&mut self, text: &Rope) {
        if let (Some(syntax), Some(theme)) = (&self.syntax, &self.theme) {
            let mut highlight = HighlightLines::new(syntax, theme);
            let mut spans: Vec<Vec<CachedSpan>> = vec![];
            let rope_str = text.to_string();
            let text_lines = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            for line in text_lines {
                if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                    spans.push(Self::to_spans(hs.clone()));
                }
            }
            self.highlight_cache = spans.clone();
        }
    }

    pub fn cache_line_numbers(&mut self, text: &Rope) {
        let line_count = text.len_lines();
        let local_line_nums = Self::line_numbers(line_count);
        self.line_num_cache = local_line_nums.clone();
    }

    pub fn cache_new_line(&mut self, text: &Rope, line_index: usize) {
        if let (Some(syntax), Some(theme)) = (&self.syntax, &self.theme) {
            let mut highlight = HighlightLines::new(syntax, theme);
            let rope_str = text
                .get_line(line_index)
                .map(|l| l.to_string())
                .unwrap_or_default();
            let mut text_line = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            if let Some(line) = &text_line.nth(0) {
                if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                    self.highlight_cache
                        .insert(line_index, Self::to_spans(hs.clone()));
                }
            }
        }
    }

    pub fn remove_cache_line(&mut self, line_index: usize) {
        let _ = self.highlight_cache.remove(line_index);
    }

    pub fn cache_current_line(&mut self, text: &Rope, line_index: usize) {
        if let (Some(syntax), Some(theme)) = (&self.syntax, &self.theme) {
            let mut highlight = HighlightLines::new(syntax, theme);
            let rope_str = text
                .get_line(line_index)
                .map(|l| l.to_string())
                .unwrap_or_default();
            let mut text_line = LinesWithEndings::from(Box::leak(Box::new(rope_str)));
            if let Some(line) = &text_line.nth(0) {
                if let Ok(hs) = highlight.highlight_line(line, &self.syntax_set.clone().unwrap()) {
                    self.highlight_cache[line_index] = Self::to_spans(hs.clone())
                }
            }
        }
    }

    fn line_numbers(line_number_count: usize) -> Vec<Vec<CachedSpan>> {
        vec![(1..line_number_count)
            .map(|l| CachedSpan {
                content: ComprString::new(&format!("{:<5}", l)),
                style: Style::default().fg(Color::Yellow),
            })
            .collect::<Vec<CachedSpan>>()]
    }

    fn get_line_offset(line_width: u16, text_area_width: u16, alignment: Alignment) -> u16 {
        match alignment {
            Alignment::Center => (text_area_width / 2).saturating_sub(line_width / 2),
            Alignment::Right => text_area_width.saturating_sub(line_width),
            Alignment::Left => 0,
        }
    }

    pub fn render_text(&self, text_area: Rect, text: Vec<Spans>, buf: &mut TuiBuffer) {
        if text_area.height < 1 {
            return;
        }

        let mut styled = text.iter().flat_map(|spans| {
            spans
                .0
                .iter()
                .flat_map(|span| span.styled_graphemes(Style::default()))
                .chain(iter::once(StyledGrapheme {
                    symbol: "\n",
                    style: Style::default(),
                }))
        });

        let mut line_composer: Box<dyn LineComposer> =
            Box::new(WordWrapper::new(&mut styled, text_area.width, false));
        let mut y = 0;
        while let Some((current_line, current_line_width)) = line_composer.next_line() {
            if y >= self.current_page {
                let mut x =
                    Self::get_line_offset(current_line_width, text_area.width, Alignment::Left);
                for StyledGrapheme { symbol, style } in current_line {
                    buf.get_mut(
                        text_area.left() + x + 1,
                        text_area.top() + y  - self.current_page,
                    )
                    .set_symbol(if symbol.is_empty() { " " } else { symbol })
                    .set_style(*style);
                    x += symbol.width() as u16;
                }
            }
            y += 1;
            if y >= (text_area.height - 2 ) + self.current_page {
                break;
            }
        }
    }

    pub fn new(change: &WindowChange) -> Self {
        if let Some(area) = change.area {
            Self {
                id: change.id,
                x_pos: change.x_pos,
                y_pos: change.y_pos,
                title: change.title.clone(),
                page_size: change.page_size,
                current_page: change.current_page,
                area: Some(area),
                y_offset: area.y + 1,
                x_offset: area.x + 4,
                ..Window::default()
            }
        } else {
            Self {
                id: change.id,
                x_pos: change.x_pos,
                y_pos: change.y_pos,
                title: change.title.clone(),
                page_size: change.page_size,
                current_page: change.current_page,
                y_offset: 1,
                x_offset: 4,
                ..Window::default()
            }
        }
    }
    pub fn cursor_x_pos(&self) -> u16 {
        let area = self.area.unwrap_or_default();
        if self.display_x_pos() >= area.right() {
            area.right() - 1
        } else {
            self.display_x_pos()
        }
    }
    pub fn display_x_pos(&self) -> u16 {
        self.x_pos + self.x_offset
    }

    pub fn cursor_y_pos(&self) -> u16 {
        let area = self.area.unwrap_or_default();
        if self.display_y_pos() >= area.bottom() {
            area.bottom() - 3
        } else {
            self.display_y_pos()
        }
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

    pub fn update(&mut self, change: WindowChange) {
        self.x_pos = change.x_pos;
        self.y_pos = change.y_pos;
        self.title = change.title;
        self.page_size = change.page_size;
        self.current_page = change.current_page;
    }
}
