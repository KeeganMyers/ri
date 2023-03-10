use crate::{
    Parser,
    token::{
        display_token::{DisplayToken, WindowChange},
        get_token_from_key, get_token_from_str, CommandToken, GetState, AppendToken,Token,
        InsertToken
    },
    ui::Term,
    Buffer, Ui, Window,
};

use actix::prelude::*;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use crossterm::{
    event::EnableMouseCapture,
    execute, terminal,
    terminal::{disable_raw_mode, enable_raw_mode, ClearType},
};
use flume::{Receiver, Sender};
use id_tree::{InsertBehavior::*, Node, Tree, TreeBuilder};
use log::trace;
use std::collections::HashMap;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tui::{backend::CrosstermBackend, layout::Direction, Terminal};
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
    pub parser: Addr<Parser>,
    pub terminal: Term,
    pub command_text: Option<String>,
    pub buffers: HashMap<Uuid, Buffer>,
    pub ui: Ui,
    pub windows: HashMap<Uuid, Window>,
    pub window_layout: Tree<Uuid>,
    pub current_window_id: Uuid,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
    pub mode: Mode,
    pub current_file: Option<String>,
}

impl Actor for App {
    type Context = Context<Self>;
}

impl App {
    pub fn get_mut_buffer(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(&self.current_buffer_id)
    }

    pub fn get_mut_window(&mut self) -> Option<&mut Window> {
        self.windows.get_mut(&self.current_window_id)
    }

    pub fn render_ui(&mut self)  {
       self.ui.draw_view_port(&self.current_window_id,self.windows.values().collect::<Vec<&Window>>(),&mut self.terminal)
    }

    pub fn set_command_mode(&mut self) {
        self.mode = Mode::Command
    }

    pub fn set_insert_mode(&mut self) {
        self.mode = Mode::Insert
    }

    pub fn set_visual_mode(&mut self) {
        self.mode = Mode::Visual;
        self.get_mut_buffer().map(|b|  {
            let idx = b.get_cursor_idx();
            b.start_select_pos = Some(idx);
        });
    }

    pub fn set_append_mode(&mut self) {
        self.mode = Mode::Append
    }

    pub fn set_normal_mode(&mut self) {
        self.mode = Mode::Normal
    }

    pub fn new(file_name: Option<String>, parser: Addr<Parser>) -> AnyHowResult<App> {
        enable_raw_mode()?;
        let _ = execute!(stdout(), terminal::Clear(ClearType::All));
        let mut stdout = stdout();
        execute!(stdout, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let mut window_layout: Tree<Uuid> = Tree::new();
        let mut buffers = HashMap::new();
        let mut windows = HashMap::new();
        let ui = Ui::new(&terminal);
        let buffer = Buffer::new(file_name.clone()).unwrap();
        let mut window = Window::new(&WindowChange {
            id: buffer.id,
            x_pos: buffer.x_pos,
            y_pos: buffer.y_pos,
            mode: buffer.mode.clone(),
            title: Some(buffer.title.clone()),
            page_size: buffer.page_size,
            current_page: buffer.current_page,
            ..WindowChange::default()
        });
        let current_buffer_id = buffer.id.clone();
        let current_window_id = window.id.clone();
        window.set_highlight();
        window.cache_window_content(&buffer.text);
        window.area = Some(ui.text_area.clone());
        let _ = window_layout.insert(Node::new(current_window_id), AsRoot);
        buffers.insert(buffer.id, buffer);
        windows.insert(current_window_id, window);
        //self.render_ui();

        Ok(Self {
            terminal,
            parser,
            windows,
            buffers,
            ui,
            current_file: file_name,
            window_layout,
            should_quit: false,
            current_buffer_id,
            current_window_id,
            mode: Mode::Normal,
            command_text: None
        })
    }

    pub fn handle_insert_token(&mut self, token: InsertToken) {
        match token {
            InsertToken::Append(chars) => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.insert_chars(&chars);
                  let change = WindowChange {
                            id: buffer.id,
                            x_pos: buffer.x_pos,
                            y_pos: buffer.y_pos,
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.cache_current_line(&buffer.text, buffer.y_pos as usize);
                        w.update(change)
                    });
                }
            }
            InsertToken::Remove => {
                if let Some(buffer) = self.get_mut_buffer() {
                    buffer.remove_char();
                  let change = WindowChange {
                            id: buffer.id,
                            x_pos: buffer.x_pos,
                            y_pos: buffer.y_pos,
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.cache_current_line(&buffer.text, buffer.y_pos as usize);
                        w.update(change)
                    });
                }
            }
            InsertToken::Esc => {
                self.set_normal_mode();
                self.get_mut_buffer().map(|b| b.start_select_pos = None);
            }
            InsertToken::Enter => {
                if let Some(buffer) = self.get_mut_buffer() {
                  buffer.insert_return();
                  let change = WindowChange {
                            id: buffer.id,
                            x_pos: buffer.x_pos,
                            y_pos: buffer.y_pos,
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.cache_new_line(&buffer.text, buffer.y_pos as usize);
                        w.cache_line_numbers(&buffer.text);
                        w.update(change)
                    });
                }
                self.render_ui();
            }
            _ => ()
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
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.update(change)
                    });
                }
                self.render_ui();
            },
            NormalToken::Down => {
                if let Some(buffer) = self.get_mut_buffer() {
                  buffer.on_down();
                  let change = WindowChange {
                            id: buffer.id,
                            x_pos: buffer.x_pos,
                            y_pos: buffer.y_pos,
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.update(change)
                    });
                }
                self.render_ui();
            },
            NormalToken::Left => {
                if let Some(buffer) = self.get_mut_buffer() {
                  buffer.on_left();
                  let change = WindowChange {
                            id: buffer.id,
                            x_pos: buffer.x_pos,
                            y_pos: buffer.y_pos,
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.update(change)
                    });
                }
                self.render_ui();
            },
            NormalToken::Right => {
                if let Some(buffer) = self.get_mut_buffer() {
                  buffer.on_right();
                  let change = WindowChange {
                            id: buffer.id,
                            x_pos: buffer.x_pos,
                            y_pos: buffer.y_pos,
                            mode: buffer.mode.clone(),
                            title: Some(buffer.title.clone()),
                            page_size: buffer.page_size,
                            current_page: buffer.current_page,
                            ..WindowChange::default()
                        };
                    self.get_mut_window().map(|w| {
                        w.update(change)
                    });
                }
                self.render_ui();
            },
            NormalToken::SwitchToCommand => {
                self.command_text = Some("".to_string());
                self.set_command_mode();
                self.render_ui();
            },
            NormalToken::SwitchToInsert => {
                self.set_insert_mode();
                self.render_ui();
            },
            NormalToken::SwitchToAppend => {
                self.set_append_mode();
                self.render_ui();
            },
            _ => ()
        }
    }

    pub fn handle_append_token(&mut self, token: AppendToken) {
        match token {
            AppendToken::Enter => {
                self.get_mut_buffer().map(|b| b.append_return());
            },
            AppendToken::Remove => {
                self.get_mut_buffer().map(|b| b.remove_char());
            },
            AppendToken::Append(chars) => {
                self.get_mut_buffer().map(|b| b.append_chars(&chars));
            },
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
                let windows = self.windows.clone();
                self.render_ui();
            },
            CommandToken::Split(_) => {
                self.set_normal_mode();
            },
            CommandToken::VerticalSplit(_) => {
                self.set_normal_mode();
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
            }
            CommandToken::Remove => {
                self.command_text = self.command_text.clone().map(|mut t| {
                    t.truncate(t.len() - 1);
                    t
                });
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
            }
            CommandToken::TabNew => (),
            CommandToken::Split(f_name) => {
                /*
                if let Some(file_name) = f_name {
                    let buffer = if let Ok(buffer) = Buffer::new(Some(file_name.trim().to_string()))
                    {
                        buffer
                    } else {
                        Buffer::new(None).unwrap()
                    };
                    let response = Ok(vec![
                        Token::Display(DisplayToken::NewWindow(
                            WindowChange {
                                id: buffer.id,
                                x_pos: buffer.x_pos,
                                y_pos: buffer.y_pos,
                                mode: buffer.mode.clone(),
                                title: Some(buffer.title.clone()),
                                page_size: buffer.page_size,
                                current_page: buffer.current_page,
                                ..WindowChange::default()
                            },
                            Some(Direction::Vertical),
                        )),
                        Token::Display(DisplayToken::SetTextLayout(Direction::Vertical)),
                        Token::Display(DisplayToken::CacheWindowContent(
                            buffer.id,
                            buffer.text.clone(),
                        )),
                        //Token::Display(DisplayToken::DrawViewPort),
                    ]);
                    self.current_buffer_id = buffer.id;
                    //self.buffers.insert(buffer.id, buffer);
                    response
                } else {
                    Ok(vec![])
                }
                */
            }
            CommandToken::VerticalSplit(f_name) => {
                /*
                if let Some(file_name) = f_name {
                    let buffer = if let Ok(buffer) = Buffer::new(Some(file_name.trim().to_string()))
                    {
                        buffer
                    } else {
                        Buffer::new(None).unwrap()
                    };
                    let response = Ok(vec![
                        Token::Display(DisplayToken::NewWindow(
                            WindowChange {
                                id: buffer.id,
                                x_pos: buffer.x_pos,
                                y_pos: buffer.y_pos,
                                mode: buffer.mode.clone(),
                                title: Some(buffer.title.clone()),
                                page_size: buffer.page_size,
                                current_page: buffer.current_page,
                                ..WindowChange::default()
                            },
                            Some(Direction::Horizontal),
                        )),
                        Token::Display(DisplayToken::SetTextLayout(Direction::Horizontal)),
                        Token::Display(DisplayToken::CacheWindowContent(
                            buffer.id,
                            buffer.text.clone(),
                        )),
                        //Token::Display(DisplayToken::DrawViewPort),
                    ]);
                    self.current_buffer_id = buffer.id;
                    //self.buffers.insert(buffer.id, buffer);
                    response
                } else {
                    Ok(vec![])
                }
                */
            }
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
            }
            CommandToken::SetMode(mode) => {
                self.mode = mode.clone();
                let _ = self.parser.try_send(Token::Command(CommandToken::SetMode(mode)));
            }
            _ => (),
        }
    }

    pub fn handle_display_token(
        &mut self,
        token: DisplayToken,
        ctx: &mut Context<Self>,
    ) {
        match token {
            DisplayToken::DrawViewPort(_, _) => {
                trace!("app attempting to handle DrawViewPort");
                self.render_ui();
            }
            _ => (),
        };
    }

    pub fn handle_token(
        &mut self,
        token: Token,
        ctx: &mut Context<Self>,
    ) {
        let _ = match token {
            Token::Command(t) => self.handle_command_token(t),
            Token::Append(t) => self.handle_append_token(t),
            Token::Normal(t) => self.handle_normal_token(t),
            Token::Insert(t) => self.handle_insert_token(t),
            Token::Display(t) => {
                self.handle_display_token(t, ctx)
            }
            _ => (),
        };
    }
}

impl Handler<Token> for App {
    type Result = ();

    fn handle(&mut self, msg: Token, ctx: &mut Context<Self>) -> Self::Result {
        let _ = self.handle_token(msg, ctx);
        ()
    }
}
