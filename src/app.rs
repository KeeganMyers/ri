use crate::util::event::{Config, Event, Events};
use crate::{
    token::{
        command_token::*,
        display_token::{DisplayToken, WindowChange},
        get_token_from_key, get_token_from_str, CommandToken, Token,
    },
    Buffer,
};
use actix::prelude::*;
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use flume::{Receiver, Sender};
use std::collections::HashMap;
use std::time::Duration;
use termion::event::Key;
use tui::layout::Direction;
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
    pub buffers: HashMap<Uuid, Buffer>,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
}

impl Actor for App {
    type Context = Context<Self>;
}

impl Handler<Quit> for App {
    type Result = ();

    fn handle(&mut self, _msg: Quit, _ctx: &mut Context<Self>) {
            let id = self.current_buffer_id;
            self.buffers.remove(&id);
            if let Some(buffer_id) = self.buffers.keys().nth(0) {
                self.current_buffer_id = *buffer_id;
            }
            if self.buffers.is_empty() {
                self.should_quit = true;
            }
            unimplemented!();
            //Ok(vec![Token::Command(CommandToken::Quit)])
    }
}


impl Handler<TabNew> for App {
    type Result = ();

    fn handle(&mut self, _msg: TabNew, _ctx: &mut Context<Self>) {
        unimplemented!()
    }
}

impl Handler<Split> for App {
    type Result = ();
    fn handle(&mut self,msg: Split, _ctx: &mut Context<Self>) {
                if let Some(file_name) = msg.f_name {
                    let buffer = if let Ok(buffer) = Buffer::new(Some(file_name.trim().to_string()))
                    {
                        buffer
                    } else {
                        Buffer::new(None).unwrap()
                    };
                    unimplemented!();
                    /*
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
                        Token::Display(DisplayToken::DrawViewPort),
                    ]);
                    */
                    self.current_buffer_id = buffer.id;
                    self.buffers.insert(buffer.id, buffer);
                    //response
                } 
                ()
    }
}


impl Handler<VerticalSplit> for App {
    type Result = ();
    fn handle(&mut self,msg: VerticalSplit, _ctx: &mut Context<Self>) {
                if let Some(file_name) = msg.f_name {
                    let buffer = if let Ok(buffer) = Buffer::new(Some(file_name.trim().to_string()))
                    {
                        buffer
                    } else {
                        Buffer::new(None).unwrap()
                    };
                    unimplemented!();
                    /*
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
                        Token::Display(DisplayToken::DrawViewPort),
                    ]);
                    */
                    self.current_buffer_id = buffer.id;
                    self.buffers.insert(buffer.id, buffer);
                    //response
                }    
    }
}


impl Handler<SetBuffer> for App {
    type Result = ();
    fn handle(&mut self,msg: SetBuffer, _ctx: &mut Context<Self>) {
                if let Some(_buffer) = self.buffers.get(&msg.id) {
                    self.current_buffer_id = msg.id;
                }
    }
}


impl Handler<Enter> for App {
    type Result = ();
    fn handle(&mut self,_msg: Enter, _ctx: &mut Context<Self>) {
                if let Some(buffer) = self.buffers.get_mut(&self.current_buffer_id) {
                    if let Some(command_text) = &buffer.command_text {
                        if let Ok(Token::Command(command)) =
                            get_token_from_str(&Mode::Command, &format!(":{}", command_text))
                        {
                            unimplemented!();
                            //return self.handle_command_token(command);
                        }
                    }
                }
    }
}

impl App {
    pub fn mode(&self) -> Mode {
        self.buffers
            .get(&self.current_buffer_id)
            .map(|b| b.mode.clone())
            .unwrap_or(Mode::Normal)
    }

    pub fn new(file_name: Option<String>) -> Result<App, std::io::Error> {
        match Buffer::new(file_name) {
            Ok(buffer) => {
                let mut buff_map = HashMap::new();
                let buff_id = buffer.id.clone();
                buff_map.insert(buffer.id, buffer);
                Ok(Self {
                    current_buffer_id: buff_id,
                    buffers: buff_map,
                    should_quit: false,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub async fn receive_tokens(
        file_name: Option<String>,
        rx: Receiver<Token>,
        tx: Sender<Token>,
    ) -> AnyHowResult<()> {
        let config = Config {
            tick_rate: Duration::from_millis(250),
            ..Config::default()
        };

        let events = Events::with_config(config.clone());
        let mut token_str = String::new();
        let mut app = App::new(file_name)?;
        let _ = tx
            .send_async(Token::Display(DisplayToken::SetHighlight))
            .await;
        if let Some(buffer) = app.buffers.get(&app.current_buffer_id) {
            let _ = tx
                .send_async(Token::Display(DisplayToken::NewWindow(
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
                    None,
                )))
                .await;
            let _ = tx
                .send_async(Token::Display(DisplayToken::SetTextLayout(
                    Direction::Horizontal,
                )))
                .await;
            let _ = tx
                .send_async(Token::Display(DisplayToken::CacheWindowContent(
                    buffer.id,
                    buffer.text.clone(),
                )))
                .await;
        }
        let _ = tx
            .send_async(Token::Display(DisplayToken::DrawViewPort))
            .await;
        loop {
            if !app.should_quit {
                let event = events.next()?;
                let mut draw_events: Vec<Token> = vec![];
                let mut app_events: Vec<Token> = vec![];
                if let Ok(token) = get_token_from_key(&app.mode(), &event) {
                    token_str.truncate(0);
                    app_events.push(token.clone());
                    draw_events.push(token.clone());
                    if let Some(buffer) = app.buffers.get_mut(&app.current_buffer_id) {
                        /*
                        if let Ok(mut buff_events) = buffer.handle_token(token.clone()) {
                            draw_events.append(&mut buff_events);
                            app_events.append(&mut buff_events);
                        }
                        */
                    }
                } else if let Event::Input(Key::Char(c)) = event {
                    token_str.push_str(&c.to_string());
                    if let Ok(token) = get_token_from_str(&app.mode(), &token_str) {
                        app_events.push(token.clone());
                        draw_events.push(token.clone());
                        token_str.truncate(0);
                        if let Some(buffer) = app.buffers.get_mut(&app.current_buffer_id) {
                            /*
                            if let Ok(mut buff_events) = buffer.handle_token(token.clone()) {
                                draw_events.append(&mut buff_events);
                                app_events.append(&mut buff_events);
                            }
                            */
                        }
                    }
                }
                unimplemented!();
                /*
                for token in app_events {
                    if let Ok(mut events) = app.handle_token(token.clone()) {
                        draw_events.append(&mut events);
                    }
                }
                for draw_event in draw_events.iter() {
                    let _ = tx.send_async(draw_event.clone()).await;
                }
                if let Ok(token) = rx.recv_timeout(Duration::from_millis(1)) {
                    let _ = app.handle_token(token);
                }
                */
            }

            if app.should_quit {
                break;
            }
        }
        Ok(())
    }
}
