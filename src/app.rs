use crate::{
    token::{
        display_token::{DisplayToken, WindowChange},
        get_token_from_key, get_token_from_str, CommandToken, GetState, Token,
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
    pub terminal: Arc<Mutex<Term>>,
    pub buffers: HashMap<Uuid, Addr<Buffer>>,
    pub ui: Option<Addr<Ui>>,
    pub windows: HashMap<Uuid, Addr<Window>>,
    pub window_layout: Tree<Uuid>,
    pub current_window_id: Uuid,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
    pub mode: Mode,
    pub current_file: Option<String>,
}

impl Actor for App {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let addr = ctx.address().recipient();
        if let Ok(ui) = Ui::new(addr.clone(), self.terminal.clone()) {
            let buffer = Buffer::new(addr, self.current_file.clone()).unwrap();
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
            let buff_id = buffer.id.clone();
            let window_id = window.id.clone();

            window.area = Some(ui.text_area.clone());
            let ui_addr = ui.start();
            let window_addr = window.start();
            let buffer_addr = buffer.clone().start();
            let _ = self.window_layout.insert(Node::new(window_id), AsRoot);
            self.buffers.insert(buffer.id, buffer_addr.clone());
            self.windows.insert(window_id, window_addr.clone());
            self.current_buffer_id = buff_id;
            self.current_window_id = window_id;

            self.ui = Some(ui_addr.clone());
            let _ = window_addr.try_send(Token::Display(DisplayToken::SetHighlight));
            let _ = window_addr.try_send(Token::Display(DisplayToken::CacheWindowContent(
                buffer.text.clone(),
            )));
            let _ = buffer_addr.try_send(Token::Command(CommandToken::SetBufferWindow(
                window_addr.clone().recipient(),
            )));
            let windows = self.windows.clone();
            async move {
                let _ = Self::render_ui(window_id, &ui_addr, &windows).await;
            }
            .into_actor(self)
            .wait(ctx)
        }
    }
}

impl App {
    pub async fn render_ui(
        current_window_id: Uuid,
        ui: &Addr<Ui>,
        windows: &HashMap<Uuid, Addr<Window>>,
    ) -> AnyHowResult<()> {
        let mut window_widgets: Vec<Window> = vec![];
        for window in windows.values() {
            if let Ok(window_widget) = window.send(GetState {}).await {
                window_widgets.push(window_widget)
            }
        }

        let _ = ui.try_send(Token::Display(DisplayToken::DrawViewPort(
            current_window_id,
            window_widgets,
        )));
        Ok(())
    }

    pub fn new(file_name: Option<String>) -> AnyHowResult<App> {
        enable_raw_mode()?;
        let _ = execute!(stdout(), terminal::Clear(ClearType::All));
        let mut stdout = stdout();
        execute!(stdout, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let window_layout: Tree<Uuid> = Tree::new();
        let terminal_arc = Arc::new(Mutex::new(terminal));
        let buff_map = HashMap::new();
        let window_map = HashMap::new();
        Ok(Self {
            terminal: terminal_arc,
            windows: window_map,
            buffers: buff_map,
            current_file: file_name,
            window_layout,
            should_quit: false,
            current_buffer_id: Uuid::new_v4(),
            current_window_id: Uuid::new_v4(),
            ui: None,
            mode: Mode::Normal,
        })
    }

    pub fn handle_command_token(&mut self, token: CommandToken) -> AnyHowResult<Vec<Token>> {
        match token {
            CommandToken::NoOp => Ok(vec![]),
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
                Ok(vec![])
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
                Ok(vec![])
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
            }
            CommandToken::SetMode(mode) => {
                self.mode = mode;
                Ok(vec![])
            }
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        }
    }

    pub fn handle_display_token(
        &mut self,
        token: DisplayToken,
        ctx: &mut Context<Self>,
    ) -> AnyHowResult<Vec<Token>> {
        trace!("calling display handle token");
        match token {
            DisplayToken::DrawViewPort(_, _) => {
                trace!("app attempting to handle DrawViewPort");
                if let Some(ui) = self.ui.clone() {
                    let windows = self.windows.clone();
                    let window_id = self.current_window_id.clone();
                    async move {
                        let _ = Self::render_ui(window_id, &ui, &windows).await;
                    }
                    .into_actor(self)
                    .wait(ctx);
                }
            }
            _ => (),
        };
        Ok(vec![])
    }

    pub fn handle_token(
        &mut self,
        token: Token,
        ctx: &mut Context<Self>,
    ) -> AnyHowResult<Vec<Token>> {
        trace!("calling handle token");
        let _ = match token {
            Token::Command(t) => self.handle_command_token(t),
            Token::Display(t) => {
                trace!("in display match");
                self.handle_display_token(t, ctx)
            }
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        };
        Ok(vec![])
    }
}

impl Handler<Token> for App {
    type Result = ();

    fn handle(&mut self, msg: Token, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(buffer) = self.buffers.get(&self.current_buffer_id) {
            let _ = buffer.try_send(msg.clone());
        }
        if let Some(window) = self.windows.get(&self.current_window_id) {
            let _ = window.try_send(msg.clone());
        }
        let _ = self.ui.as_ref().and_then(|ui| Some(ui.send(msg.clone())));
        let _ = self.handle_token(msg, ctx);
        ()
    }
}
