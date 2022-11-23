use crate::{
    token::{CommandToken, get_token_from_key, get_token_from_str, Token,display_token::{WindowChange,DisplayToken}},
    Buffer, Window,
};

use std::collections::HashMap;
use termion::event::Key;
use std::{ time::Duration};
use crate::util::event::{Config, Event, Events};
use anyhow::{Error as AnyHowError, Result as AnyHowResult};
use flume::{Sender,Receiver};
use ropey::Rope;
use syntect::{
    highlighting::ThemeSet,
    parsing::{SyntaxReference, SyntaxSet},
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
    pub buffers: HashMap<Uuid,Buffer>,
    pub current_buffer_id: Uuid,
    pub should_quit: bool,
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
                buff_map.insert(buffer.id,buffer);
                Ok(Self {
                    current_buffer_id: buff_id,
                    buffers: buff_map,
                    should_quit: false,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn handle_command_token(&mut self, token: CommandToken) -> AnyHowResult<Vec<Token>> {
        let _ = match token {
            CommandToken::Quit => {
                self.should_quit = true;
                Ok(())
            },
            CommandToken::TabNew => {
                    Ok(())
            },
            CommandToken::VerticalSplit(f_name) => {
                    if let Some(file_name) = f_name {
                        let buffer = if let Ok(buffer) = Buffer::new(Some(file_name)) {
                            buffer
                        } else {
                            Buffer::new(None).unwrap()
                        };
                        self.buffers.insert(buffer.id,buffer);
                        //add NewVerticalWindow(WindowChange),
                    } else {
                        //add NewVerticalWindow(WindowChange),
                    }
                    //set current buffer to normal
                    Ok(())
            },
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        };
        Ok(vec![])
    }

    pub fn handle_token(&mut self, token: Token) -> AnyHowResult<Vec<Token>> {
        match token {
            //Token::Append(t) => self.handle_append_token(t),
            Token::Command(t) => self.handle_command_token(t),
            //Token::Insert(t) => self.handle_insert_token(t),
            //Token::Normal(t) => self.handle_normal_token(t),
            //Token::Operator(t) => self.handle_operator_token(t),
            //Token::Range(t) => self.handle_range_token(t),
            _ => Err(AnyHowError::msg("No Tokens Found".to_string())),
        }
    }

    pub async fn receive_tokens(
        file_name: Option<String>,
        rx: Receiver<Token>,
        tx: Sender<Token>) -> AnyHowResult<()>
    {
        let config = Config {
            tick_rate: Duration::from_millis(250),
            ..Config::default()
        };

        let events = Events::with_config(config.clone());
        let mut token_str = String::new();
        let mut app = App::new(file_name)?;
        let _ = tx.send_async(Token::Display(DisplayToken::SetHighlight)).await;
        if let Some(buffer) = app.buffers.get(&app.current_buffer_id) {
            let _ = tx.send_async(Token::Display(DisplayToken::NewVerticalWindow(
                            WindowChange {
                                id: buffer.id,
                                x_pos: buffer.x_pos,
                                y_pos: buffer.y_pos,
                                mode: buffer.mode.clone(),
                                title: Some(buffer.title.clone()),
                                page_size: buffer.page_size,
                                current_page: buffer.current_page,
                                ..WindowChange::default()
                            }
                        ))).await; 
            let _ = tx.send_async(Token::Display(DisplayToken::CacheWindowContent(buffer.id,buffer.text.clone()))).await;
        }
        let _ = tx.send_async(Token::Display(DisplayToken::DrawViewPort)).await;
        loop {
            if !app.should_quit {
                let event = events.next()?;
                let mut draw_events: Vec<Token> = vec![];
                    if let Ok(token) = get_token_from_key(&app.mode(), &event) {
                        token_str.truncate(0);
                        draw_events.push(token.clone());
                        let _ = app.handle_token(token.clone());
                        if let Some(buffer) = app.buffers.get_mut(&app.current_buffer_id) {
                            if let Ok(mut buff_events) = buffer.handle_token(token.clone()) {
                                draw_events.append(&mut buff_events);
                            }

                        }
                    } else if let Event::Input(Key::Char(c)) = event {
                        token_str.push_str(&c.to_string());
                        if let Ok(token) = get_token_from_str(&app.mode(), &token_str) {
                            token_str.truncate(0);
                            draw_events.push(token.clone());
                            let _ = app.handle_token(token.clone());
                            if let Some(buffer) = app.buffers.get_mut(&app.current_buffer_id) {
                                if let Ok(mut buff_events) = buffer.handle_token(token.clone()) {
                                    draw_events.append(&mut buff_events);
                                }
                            }
                        }
                    }
                    for draw_event in draw_events.iter() {
                        let _ = tx.send_async(draw_event.clone()).await;
                    }
            }

            if app.should_quit {
                break;
            }
        }
        Ok(())
    }
}
