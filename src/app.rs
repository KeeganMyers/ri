use crate::{
    token::{
        display_token::{DisplayToken, WindowChange},
        get_token_from_str, AppendToken, CommandToken, InsertToken, NormalToken, Token,
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
use id_tree::{InsertBehavior::*, Node, Tree};
use log::trace;
use std::collections::HashMap;
use std::io::stdout;
use tui::{backend::CrosstermBackend, Terminal, layout::{Direction,Rect}};
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
    pub window_layout: Tree<Rect>,
    pub current_window_id: Uuid,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
    pub mode: Mode,
    pub current_file: Option<String>,
}

impl App {
    pub fn get_mut_buffer(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(&self.current_buffer_id)
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
            &self.current_file,
            &self.mode,
            self.get_buffer().map(|b| (b.x_pos,b.y_pos)),
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
        let mut window_layout: Tree<Rect> = Tree::new();
        let mut buffers = HashMap::new();
        let mut windows = HashMap::new();
        let ui = Ui::new(&mut terminal);
        let root_node = window_layout.insert(Node::new(ui.text_area), AsRoot)?;
        let buffer = Buffer::new(file_name.clone())?;
        let mut window = Window::new(&WindowChange {
            id: buffer.id,
            x_pos: buffer.x_pos,
            y_pos: buffer.y_pos,
            area: Some(ui.text_area.clone()),
            title: Some(buffer.title.clone()),
            page_size: buffer.page_size,
            node_id: Some(root_node.clone()),
            current_page: buffer.current_page,
            ..WindowChange::default()
        });
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
            current_file: file_name,
            window_layout,
            should_quit: false,
            current_buffer_id,
            current_window_id,
            mode: Mode::Normal,
            command_text: None,
        })
    }

    pub fn new_split(&mut self,file_name: Option<String>,direction: Direction) -> AnyHowResult<()> {
        if let Some(current_window) = self.get_window().clone() {
            if let Some(node_id) = current_window.node_id.clone() {
                let buffer = Buffer::new(file_name.clone())?;

                if let [split1,split2,..] = self.ui.split_ui(&current_window,direction)[..] {
                let node_id_1 = self.window_layout.insert(Node::new(split1), UnderNode(&node_id))?;
                let node_id_2 = self.window_layout.insert(Node::new(split2), UnderNode(&node_id))?;
                let mut window = Window::new(&WindowChange {
                    id: buffer.id,
                    area: Some(split2),
                    x_pos: buffer.x_pos,
                    y_pos: buffer.y_pos,
                    title: Some(buffer.title.clone()),
                    page_size: buffer.page_size,
                    node_id: Some(node_id_2.clone()),
                    current_page: buffer.current_page,
                    ..WindowChange::default()
                });
                let current_buffer_id = buffer.id.clone();
                let current_window_id = window.id.clone();
                window.set_highlight();
                window.cache_window_content(&buffer.text);
                self.current_file = buffer.file_path.clone();
                self.buffers.insert(current_buffer_id, buffer);
                self.windows.insert(current_window_id, window);
                self.get_mut_window().map(|w| {
                    w.node_id = Some(node_id_1);
                    w.area = Some(split1);
                    w.x_offset = split1.x + 4;
                    w.y_offset = split1.y + 1;
                });
                self.current_buffer_id = current_buffer_id;
                self.current_window_id = current_window_id;
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

    pub fn handle_normal_token(&mut self, token: NormalToken) {
        match token {
            NormalToken::Up => {
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
            NormalToken::Down => {
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
            NormalToken::Left => {
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
            NormalToken::Right => {
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
            NormalToken::Last => {
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
            NormalToken::LastNonBlank => {
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
            NormalToken::First => {
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
            NormalToken::FirstNonBlank => {
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
            NormalToken::StartWord => {
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
            CommandToken::Split(file_name) => {
                let _ = self.new_split(file_name,Direction::Vertical);
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::VerticalSplit(file_name) => {
                let _ = self.new_split(file_name,Direction::Horizontal);
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
            CommandToken::Quit => {
                let id = self.current_buffer_id;
                self.buffers.remove(&id);
                if let Some(buffer_id) = self.buffers.keys().nth(0) {
                    self.current_buffer_id = *buffer_id;
                }
                if self.buffers.is_empty() {
                    self.should_quit = true;
                }
                self.set_normal_mode();
                self.render_ui();
            }
            CommandToken::TabNew => (),
            CommandToken::Enter => {
                if let Some(command_text) = &self.command_text {
                    if let Ok(Token::Command(command)) =
                        get_token_from_str(&Mode::Command, &format!(":{}", command_text))
                    {
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

    pub fn handle_token(&mut self, token: Token) {
        let _ = match token {
            Token::Command(t) => self.handle_command_token(t),
            Token::Append(t) => self.handle_append_token(t),
            Token::Normal(t) => self.handle_normal_token(t),
            Token::Insert(t) => self.handle_insert_token(t),
            Token::Display(t) => self.handle_display_token(t),
            _ => (),
        };
    }
}

