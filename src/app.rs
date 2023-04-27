use crate::{
    token::{
        display_token::{DisplayToken, WindowChange},
        get_token_from_chars, AppendToken, CommandToken, InsertToken, NormalToken, OperatorToken,
        MotionToken,
        Token,
    },
    ui::Term,
    Buffer, Ui, Window,
};

use anyhow::Result as AnyHowResult;
use crossterm::{
    event::EnableMouseCapture,
    execute, terminal,
    terminal::{enable_raw_mode, ClearType},
};
use id_tree::{InsertBehavior::*, Node, NodeId, RemoveBehavior::*, Tree};
use log::trace;
use std::collections::HashMap;
use std::io::stdout;
use tui::{
    backend::CrosstermBackend,
    layout::{Direction, Rect},
    Terminal,
};
use uuid::Uuid;

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Mode {
    Insert,
    Append,
    Visual,
    Normal,
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

pub struct App {
    pub terminal: Term,
    pub command_text: Option<String>,
    pub buffers: HashMap<Uuid, Buffer>,
    pub ui: Ui,
    pub windows: HashMap<Uuid, Window>,
    pub window_layout: Tree<(Rect, Uuid)>,
    pub current_window_id: Uuid,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
    pub mode: Mode,
}

impl App {
    pub fn get_mut_buffer(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(&self.current_buffer_id)
    }

    pub fn reorder_windows(&mut self) {
        for (idx, window) in self.windows.values_mut().enumerate() {
            window.order = idx
        }
    }

    pub fn get_buffer(&self) -> Option<&Buffer> {
        self.buffers.get(&self.current_buffer_id)
    }

    pub fn get_window(&self) -> Option<&Window> {
        self.windows.get(&self.current_window_id)
    }

    pub fn get_mut_window(&mut self) -> Option<&mut Window> {
        self.windows.get_mut(&self.current_window_id)
    }

    pub fn get_mut_pair(&mut self) -> (Option<&mut Window>, Option<&mut Buffer>) {
        (
            self.windows.get_mut(&self.current_window_id),
            self.buffers.get_mut(&self.current_buffer_id),
        )
    }

    pub fn render_ui(&mut self) {
        self.ui.draw_view_port(
            &self.current_window_id,
            &self.mode,
            self.get_buffer().map(|b| (b.x_pos, b.y_pos)),
            &self.command_text,
            self.windows.values().collect::<Vec<&Window>>(),
            &mut self.terminal,
        )
    }

    pub fn set_command_mode(&mut self) {
        self.mode = Mode::Command
    }

    pub fn set_insert_mode(&mut self) {
        self.mode = Mode::Insert
    }

    #[allow(dead_code)]
    pub fn set_visual_mode(&mut self) {
        self.mode = Mode::Visual;
        self.get_mut_buffer().map(|b| {
            let idx = b.get_cursor_idx();
            b.start_select_pos = Some(idx);
        });
    }

    pub fn set_append_mode(&mut self) {
        self.mode = Mode::Append
    }

    pub fn set_normal_mode(&mut self) {
        self.command_text = Some("".to_string());
        self.mode = Mode::Normal
    }

    pub fn new(file_name: Option<String>) -> AnyHowResult<App> {
        enable_raw_mode()?;
        let _ = execute!(stdout(), terminal::Clear(ClearType::All));
        let mut stdout = stdout();
        execute!(stdout, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let mut window_layout: Tree<(Rect, Uuid)> = Tree::new();
        let mut buffers = HashMap::new();
        let mut windows = HashMap::new();
        let ui = Ui::new(&mut terminal);
        let buffer = Buffer::new(file_name.clone())?;
        let mut window = Window::new(&WindowChange {
            id: buffer.id,
            x_pos: buffer.x_pos,
            y_pos: buffer.y_pos,
            area: Some(ui.text_area.clone()),
            title: Some(buffer.title.clone()),
            page_size: buffer.page_size,
            current_page: buffer.current_page,
            ..WindowChange::default()
        });
        let _root_node = window_layout.insert(Node::new((ui.text_area, window.id)), AsRoot)?;
        let current_buffer_id = buffer.id.clone();
        let current_window_id = window.id.clone();
        window.set_highlight();
        window.cache_window_content(&buffer.text);
        buffers.insert(buffer.id, buffer);
        windows.insert(current_window_id, window);

        Ok(Self {
            terminal,
            windows,
            buffers,
            ui,
            window_layout,
            should_quit: false,
            current_buffer_id,
            current_window_id,
            mode: Mode::Normal,
            command_text: None,
        })
    }

    pub fn new_split(
        &mut self,
        file_name: Option<String>,
        direction: Direction,
    ) -> AnyHowResult<()> {
        if let Some(current_window) = self.get_window().clone() {
            if let Ok(Some(current_node_id)) = self.get_current_node_id() {
                let buffer = Buffer::new(file_name.clone())?;

                if let [split1, split2, ..] = self.ui.split_ui(&current_window, direction)[..] {
                    let mut window = Window::new(&WindowChange {
                        id: buffer.id,
                        area: Some(split2),
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    });
                    let _ = self.window_layout.insert(
                        Node::new((split1, current_window.id)),
                        UnderNode(&current_node_id),
                    )?;
                    let _ = self
                        .window_layout
                        .insert(Node::new((split2, window.id)), UnderNode(&current_node_id))?;
                    let current_buffer_id = buffer.id.clone();
                    let current_window_id = window.id.clone();
                    window.set_highlight();
                    window.cache_window_content(&buffer.text);
                    self.buffers.insert(current_buffer_id, buffer);
                    self.windows.insert(current_window_id, window);
                    let _ = self
                        .window_layout
                        .get_mut(&current_node_id)
                        .map(|n| (n.data().0, Uuid::new_v4()));
                    self.get_mut_window().map(|w| {
                        w.area = Some(split1);
                        w.x_offset = split1.x + 4;
                        w.y_offset = split1.y + 1;
                    });
                    self.current_buffer_id = current_buffer_id;
                    self.current_window_id = current_window_id;
                    self.reorder_windows();
                }
            }
        }
        Ok(())
    }

    pub fn handle_insert_token(&mut self, token: InsertToken) {
        match token {
            InsertToken::Append(chars) => {
                if let (Some(window), Some(buffer)) = self.get_mut_pair() {
                    buffer.insert_chars(&chars);
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    window.cache_current_line(&buffer.text, buffer.y_pos.clone() as usize);
                    window.update(change);
                }
            }
            InsertToken::Remove => {
                if let (Some(window), Some(buffer)) = self.get_mut_pair() {
                    buffer.remove_char();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    window.cache_current_line(&buffer.text, buffer.y_pos as usize);
                    window.update(change)
                }
            }
            InsertToken::Esc => {
                self.set_normal_mode();
                self.get_mut_buffer().map(|b| b.start_select_pos = None);
            }
            InsertToken::Enter => {
                if let (Some(window), Some(buffer)) = self.get_mut_pair() {
                    buffer.insert_return();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    window.cache_new_line(&buffer.text, buffer.y_pos as usize);
                    window.cache_line_numbers(&buffer.text);
                    window.update(change)
                }
                self.render_ui();
            }
        }
    }

    pub fn get_current_node_id(&self) -> AnyHowResult<Option<NodeId>> {
        let current_window_id = self.current_window_id;
        if let Some(root_node_id) = self.window_layout.root_node_id() {
            for (child, id) in self
                .window_layout
                .traverse_pre_order(root_node_id)?
                .zip(self.window_layout.traverse_pre_order_ids(root_node_id)?)
            {
                if child.data().1 == current_window_id {
                    return Ok(Some(id));
                }
            }
        }
        Ok(None)
    }

    pub fn get_parent_node(&self, current_node: NodeId) -> Option<((Rect, Uuid), NodeId)> {
        if let Ok(node) = self.window_layout.get(&current_node) {
            if let Some(parent_node_id) = node.parent() {
                if let Ok(parent_node) = self.window_layout.get(parent_node_id) {
                    return Some((parent_node.data().clone(), parent_node_id.clone()));
                }
            }
        }
        None
    }

    pub fn get_sibling_node(&self, parent_node_id: NodeId) -> AnyHowResult<Option<NodeId>> {
        for (child, id) in self
            .window_layout
            .children(&parent_node_id)?
            .zip(self.window_layout.children_ids(&parent_node_id)?)
        {
            if child.data().1 != self.current_window_id {
                return Ok(Some(id.clone()));
            }
        }
        Ok(None)
    }

    fn on_quit(&mut self) {
        let id = self.current_buffer_id;
        self.buffers.remove(&id);
        if self.buffers.is_empty() {
            let _ = execute!(stdout(), terminal::Clear(ClearType::All));
            self.should_quit = true;
        } else {
            if let Ok(Some(current_node)) = self.get_current_node_id() {
                if let Some(((parent_rect, _), parent_node_id)) =
                    self.get_parent_node(current_node.clone())
                {
                    if let Ok(Some(sibling_node_id)) = self.get_sibling_node(parent_node_id.clone())
                    {
                        if let Ok((_, sibling_window_id)) = self
                            .window_layout
                            .get(&sibling_node_id)
                            .map(|n| n.data().clone())
                        {
                            let _ = self
                                .window_layout
                                .get_mut(&parent_node_id)
                                .map(|n| (n.data().0, sibling_window_id));
                            let _ = self
                                .window_layout
                                .remove_node(current_node.clone(), LiftChildren);
                            let _ = self
                                .window_layout
                                .remove_node(sibling_node_id.clone(), LiftChildren);
                            self.windows.remove(&id);
                            self.current_window_id = sibling_window_id;
                            self.current_buffer_id = sibling_window_id;
                            self.get_mut_window().map(|w| w.area = Some(parent_rect));
                            self.reorder_windows();
                        }
                    }
                }
            }
        }
        self.set_normal_mode();
        self.render_ui();
    }

    pub fn handle_normal_token(&mut self, token: NormalToken) {
        match token {
            NormalToken::SwitchToCommand => {
                self.command_text = Some("".to_string());
                self.set_command_mode();
                self.render_ui();
            }
            NormalToken::SwitchToInsert => {
                self.set_insert_mode();
                self.render_ui();
            }
            NormalToken::SwitchToAppend => {
                self.set_append_mode();
                self.render_ui();
            }
            NormalToken::AddNewLineBelow => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.add_newline_below();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            NormalToken::AddNewLineAbove => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.add_newline_above();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            NormalToken::Paste => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.paste_text();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            NormalToken::Undo => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.undo();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            NormalToken::Redo => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.redo();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            NormalToken::DeleteLine => {
                if let (Some(window), Some(buffer)) = self.get_mut_pair() {
                    let removed_line_index = buffer.y_pos;
                    buffer.delete_line();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    let _ = window.remove_cache_line(removed_line_index as usize);
                    window.cache_line_numbers(&buffer.text);
                    window.update(change)
                }

                self.render_ui();
            }
            NormalToken::Visual => {
                self.set_visual_mode();
                self.render_ui();
            }
            NormalToken::VisualLine => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.select_line();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            NormalToken::SetWindow(window_order) => {
                log::debug!("in set window {:?}", window_order);
                if let Some(window) = self
                    .windows
                    .values()
                    .filter(|w| w.order == window_order)
                    .nth(0)
                {
                    self.current_window_id = window.id;
                    self.current_buffer_id = window.id;
                }
                self.render_ui();
            }
            _ => (),
        }
    }

    pub fn handle_append_token(&mut self, token: AppendToken) {
        match token {
            AppendToken::Enter => {
                self.get_mut_buffer().map(|b| b.append_return());
            }
            AppendToken::Remove => {
                self.get_mut_buffer().map(|b| b.remove_char());
                self.render_ui();
            }
            AppendToken::Append(chars) => {
                self.get_mut_buffer().map(|b| b.append_chars(&chars));
                self.render_ui();
            }
            AppendToken::Esc => {
                self.get_mut_buffer().map(|b| b.start_select_pos = None);
                self.set_normal_mode();
            }
        }
    }

    pub fn handle_command_token(&mut self, token: CommandToken) {
        match token {
            CommandToken::Write => {
                self.get_mut_buffer().and_then(|b| b.on_save().ok());
                self.render_ui();
            }
            CommandToken::GoToLine(line_number) => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.move_to_line_number(line_number);
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::YankLines(start_idx, end_idx) => {
                self.get_buffer().map(|b| b.yank_lines(start_idx, end_idx));
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::DeleteLines(start_idx, end_idx) => {
                let mut start = start_idx;
                let mut end = end_idx;
                if start_idx > end_idx {
                    start = end_idx;
                    end = start_idx;
                }
                if let (Some(window), Some(buffer)) = self.get_mut_pair() {
                    buffer.set_states();
                    for idx in start..end {
                        buffer.move_to_line_number(idx);
                        buffer.delete_line_direct();
                        let _ = window.remove_cache_line(idx - 1 as usize);
                    }

                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    window.cache_line_numbers(&buffer.text);
                    window.update(change);

                    self.render_ui();
                    self.set_normal_mode();
                    self.render_ui();
                }
            }
            CommandToken::Split(file_name) => {
                let _ = self.new_split(file_name, Direction::Vertical);
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::VerticalSplit(file_name) => {
                let _ = self.new_split(file_name, Direction::Horizontal);
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::Esc => {
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::Append(chars) => {
                self.command_text = self.command_text.clone().map(|mut t| {
                    t.push_str(&chars);
                    t
                });
                self.render_ui();
            }
            CommandToken::Remove => {
                self.command_text = self.command_text.clone().map(|mut t| {
                    t.truncate(t.len() - 1);
                    t
                });
                self.render_ui();
            }
            CommandToken::NoOp => (),
            CommandToken::Quit => self.on_quit(),
            CommandToken::TabNew => (),
            CommandToken::Enter => {
                if let Some(command_text) = &self.command_text {
                    if let Ok(Token::Command(command)) = get_token_from_chars(
                        &Mode::Command,
                        &format!(":{}", command_text).chars().collect::<Vec<char>>(),
                    ) {
                        return self.handle_command_token(command);
                    }
                }
            }
            CommandToken::SetBuffer(id) => {
                if let Some(_buffer) = self.buffers.get(&id) {
                    self.current_buffer_id = id;
                }
                self.render_ui();
            }
            CommandToken::SetMode(mode) => {
                self.mode = mode.clone();
                self.render_ui();
            }
            _ => (),
        }
    }

    pub fn handle_display_token(&mut self, token: DisplayToken) {
        match token {
            DisplayToken::DrawViewPort => {
                trace!("app attempting to handle DrawViewPort");
                self.render_ui();
            }
            _ => (),
        };
    }

    pub fn handle_operator_token(&mut self, _token: OperatorToken) {
        unimplemented!()
    }

    pub fn handle_operator_token_range(&mut self, token: OperatorToken, start_range: usize, end_range: usize) {
        match token {
            OperatorToken::Yank => {
                self.get_buffer().map(|b| b.yank_line_range(start_range,end_range));
            },
            OperatorToken::Delete => {
                self.get_mut_buffer().map(|b| b.delete_line_range(start_range,end_range));
            },
            OperatorToken::Change => {
                unimplemented!()
            },
            OperatorToken::Indent => {
                unimplemented!()
            },
            OperatorToken::UnIndent => {
                unimplemented!()
            },
            OperatorToken::Uppercase => {
                unimplemented!()
            },
            OperatorToken::Lowercase => {
                unimplemented!()
            },
            OperatorToken::ToggleCase => {
                unimplemented!()
            },
            OperatorToken::Esc => {
                self.set_normal_mode();
                self.get_mut_buffer().map(|b| b.start_select_pos = None);
            },
            OperatorToken::Remove => {
                unimplemented!()
            },
            _ => ()
        }
    }

    pub fn handle_motion_token_range(&mut self, token: MotionToken) -> Option<(usize,usize)> {
        match token {
            MotionToken::Up => self.get_buffer().map(|b| b.on_up_range()),
            MotionToken::Down => self.get_buffer().map(|b| b.on_down_range()),
            MotionToken::Left => self.get_buffer().map(|b| b.on_left_range()),
            MotionToken::Right => self.get_buffer().map(|b| b.on_right_range()),
            _ => None
        }
    }

    pub fn handle_motion_token(&mut self, token: MotionToken) {
        match token {
            MotionToken::Up => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.on_up();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            MotionToken::Down => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.on_down();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            MotionToken::Left => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.on_left();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            MotionToken::Right => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.on_right();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }
                self.render_ui();
            }
            MotionToken::Last => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = (buffer.current_line_len() - 2) as u16;
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::LastNonBlank => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = (buffer.current_line_len() - 2) as u16;
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::First => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = 0 as u16;
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::FirstLine => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.move_to_first_line();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::LastLine => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.move_to_last_line();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::FirstNonBlank => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = 0 as u16;
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::StartWord => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = buffer.find_next_word();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::BackWord => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = buffer.find_last_word();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            MotionToken::EndWord => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.x_pos = buffer.end_current_word();
                    let change = WindowChange {
                        id: buffer.id,
                        x_pos: buffer.x_pos,
                        y_pos: buffer.y_pos,
                        title: Some(buffer.title.clone()),
                        page_size: buffer.page_size,
                        current_page: buffer.current_page,
                        ..WindowChange::default()
                    };
                    self.get_mut_window().map(|w| w.update(change));
                }

                self.render_ui();
            }
            _ => ()
        }
    }

    pub fn handle_tokens(&mut self, parsed_tokens: Vec<Token>) {
        let mut tokens = parsed_tokens.clone();
        let operator_idx = tokens.iter().position(|t| {
            if let Token::Operator(_) = *t {
                true
            } else {
            false
            }
        });
        let operator = if let Some(idx) = operator_idx {
            Some(tokens.swap_remove(idx))
        } else {
            None
        };
        for token in tokens {
            match token {
                Token::Command(t) => self.handle_command_token(t),
                Token::Append(t) => self.handle_append_token(t),
                Token::Normal(t) => self.handle_normal_token(t),
                Token::Insert(t) => self.handle_insert_token(t),
                Token::Operator(t) => self.handle_operator_token(t),
                Token::Display(t) => self.handle_display_token(t),
                Token::Motion(t) if operator_idx.is_some() =>  {
                    if let (Some((start_range,end_range)),Some(Token::Operator(o))) = (self.handle_motion_token_range(t),&operator) {
                        self.handle_operator_token_range(o.clone(),start_range,end_range);
                    }
                    ()
                },
                Token::Motion(t)  => self.handle_motion_token(t),
            }
        }
    }
}
