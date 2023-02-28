use crate::{
    token::{
        display_token::{DisplayToken, WindowChange},
        get_token_from_key, get_token_from_str, CommandToken, Token,
        GetState
    },
    Ui,Window, ui::Term,
    Buffer,
};

use std::sync::{Mutex,Arc};
use actix::prelude::*;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use flume::{Receiver, Sender};
use std::collections::HashMap;
use std::time::Duration;
use tui::{Terminal,layout::Direction,backend::CrosstermBackend};
use crossterm::{event::EnableMouseCapture,execute,terminal,terminal::{enable_raw_mode,disable_raw_mode,ClearType}};
use std::io::stdout;
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
    pub terminal: Arc<Mutex<Term>>,
    pub buffers: HashMap<Uuid, Addr<Buffer>>,
    pub ui: Option<Addr<Ui>>,
    pub windows: HashMap<Uuid, Addr<Window>>,
    pub current_window_id: Uuid,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
    pub mode: Mode,
    pub current_file: Option<String>
}

impl Actor for App {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = ctx.address().recipient();
        if let Ok(ui) =  Ui::new(addr,self.terminal.clone()) {
            let buffer = Buffer::new(self.current_file.clone()).unwrap();
            let mut window = Window::new(&buffer);
            let buff_id = buffer.id.clone();
            let window_id = window.id.clone();

            window.area = Some(ui.text_area.clone());
            let ui_addr = ui.start();
            self.buffers.insert(buffer.id, buffer.start());
            self.windows.insert(window.id, window.start());
            self.current_buffer_id = buff_id;
            self.current_window_id = window_id;

            self.ui = Some(ui_addr.clone());
            let _ = ui_addr.try_send(Token::Display(DisplayToken::SetHighlight));
            let _ = ui_addr.try_send(Token::Display(DisplayToken::SetTextLayout(Direction::Horizontal)));
            
            let windows = self.windows.clone();
            async move {
            let _ = Self::render_ui(&ui_addr,&windows).await;
            }.into_actor(self)
            .wait(ctx)
        }
    } 
}

impl App {
    pub async fn render_ui(ui: &Addr<Ui>, windows: &HashMap<Uuid,Addr<Window>>) -> AnyHowResult<()> {
            let mut window_widgets: Vec<Window> = vec![];
            for window in windows.values() {
                if let Ok(window_widget) = window.send(GetState {}).await {
                  window_widgets.push(window_widget)
                }
            }

            let _ = ui.try_send(Token::Display(DisplayToken::DrawViewPort(window_widgets)));
        Ok(())
    }

    pub fn new(file_name: Option<String>) -> AnyHowResult<App> {
        enable_raw_mode()?;
        let _ = execute!(stdout(), terminal::Clear(ClearType::All));
       let mut stdout = stdout();
        execute!(stdout, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let  terminal_arc = Arc::new(Mutex::new(terminal));
        let buff_map = HashMap::new();
        let window_map = HashMap::new();
        Ok(Self {
            terminal: terminal_arc,
            windows: window_map,
            buffers: buff_map,
            current_file: file_name,
            should_quit: false,
            current_buffer_id: Uuid::new_v4(),
            current_window_id: Uuid::new_v4(),
            ui: None,
            mode: Mode::Normal
        })
    }

    pub fn handle_command_token(&mut self, token: CommandToken) -> AnyHowResult<Vec<Token>> {
        match token {
            CommandToken::NoOp => {
                Ok(vec![])
            },
            CommandToken::Quit => {
                let id = self.current_buffer_id;
                self.buffers.remove(&id);
                if let Some(buffer_id) = self.buffers.keys().nth(0) {
                    self.current_buffer_id = *buffer_id;
                }
                if self.buffers.is_empty() {
                    self.should_quit = true;
                }
                Ok(vec![Token::Command(CommandToken::Quit)])
            }
            CommandToken::TabNew => Ok(vec![]),
            CommandToken::Split(f_name) => {
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
            }
            CommandToken::VerticalSplit(f_name) => {
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
            }
            CommandToken::Enter => {
                if let Some(buffer) = self.buffers.get_mut(&self.current_buffer_id) {
                    /*
                    if let Some(command_text) = &buffer.command_text {
                        if let Ok(Token::Command(command)) =
                            get_token_from_str(&Mode::Command, &format!(":{}", command_text))
                        {
                            return self.handle_command_token(command);
                        }
                    }
                    */
                }
                Ok(vec![])
            }
            CommandToken::SetBuffer(id) => {
                if let Some(_buffer) = self.buffers.get(&id) {
                    self.current_buffer_id = id;
                }
                Ok(vec![])
            },
            CommandToken::SetMode(mode) => {
                self.mode = mode;
                Ok(vec![])
            },
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        }
    }

    pub fn handle_token(&mut self, token: Token) -> AnyHowResult<Vec<Token>> {
        match token {
            Token::Command(t) => self.handle_command_token(t),
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        }
    }
}

impl Handler<Token> for App {
    type Result = ();

    fn handle(&mut self, msg: Token , _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(buffer) = self.buffers.get(&self.current_buffer_id) {
            let _ = buffer.send(msg.clone());
        }
        if let Some(window) = self.windows.get(&self.current_window_id) {
            let _ = window.send(msg.clone());
        }
        let _ = self.ui.as_ref().and_then(|ui| Some(ui.send(msg.clone())));
        let _ = self.handle_token(msg);
        ()
    }
}
