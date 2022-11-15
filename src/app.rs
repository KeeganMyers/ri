use crate::util::event::{Config, Event};
use crate::{
    token::{CommandToken, NormalToken, Token},
    Buffer, Window,
};
use anyhow::Result as AnyhowResult;
use ropey::Rope;
use syntect::{
    highlighting::ThemeSet,
    parsing::{SyntaxReference, SyntaxSet},
};
use termion::event::Key;
use uuid::Uuid;

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub enum Command {
    #[serde(alias = "q")]
    Quit,
    #[serde(alias = "w")]
    Write,
    TabNew,
    #[serde(alias = "vs")]
    VerticalSplit,
    #[serde(alias = "sp")]
    Split,
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Mode {
    Insert,
    Append,
    Visual,
    Normal,
    Command,
}

pub struct App {
    // pub current_tab: u8,
    // pub tabs: Option<Vec<Tab>>,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub syntax: SyntaxReference,
    pub current_window: u8,
    pub windows: Vec<Window>,
    pub buffers: Vec<Buffer>,
    pub current_buffer: usize,
    pub should_quit: bool,
}

impl App {
    pub fn on_save(&mut self) -> Result<(), std::io::Error> {
        if let Some(buffer) = self.buffers.get_mut(self.current_buffer) {
            return buffer.on_save();
        }
        Ok(())
    }

    pub fn x_pos(&self) -> u16 {
        self.buffers
            .get(self.current_buffer)
            .map(|b| b.x_pos)
            .unwrap_or_default()
    }
    pub fn display_x_pos(&self) -> u16 {
        self.buffers
            .get(self.current_buffer)
            .map(|b| b.x_pos + b.x_offset)
            .unwrap_or_default()
    }

    pub fn y_pos(&self) -> u16 {
        self.buffers
            .get(self.current_buffer)
            .map(|b| b.y_pos)
            .unwrap_or_default()
    }

    pub fn display_y_pos(&self) -> u16 {
        self.buffers
            .get(self.current_buffer)
            .map(|b| (b.y_pos + b.y_offset) - b.current_page)
            .unwrap_or_default()
    }

    pub fn set_y_offset(&mut self, offset: u16) {
        if let Some(buffer) = self.buffers.get_mut(self.current_buffer) {
            buffer.y_offset = offset
        }
    }

    pub fn set_x_offset(&mut self, offset: u16) {
        if let Some(buffer) = self.buffers.get_mut(self.current_buffer) {
            buffer.x_offset = offset
        }
    }

    pub fn mode(&self) -> Mode {
        self.buffers
            .get(self.current_buffer)
            .map(|b| b.mode.clone())
            .unwrap_or(Mode::Normal)
    }

    pub fn current_page(&self) -> Option<u16> {
        self.buffers
            .get(self.current_buffer)
            .map(|b| b.current_page)
    }

    pub fn command_text(&self) -> Option<String> {
        self.buffers
            .get(self.current_buffer)
            .and_then(|b| b.command_text.clone())
    }

    pub fn title(&self) -> Option<String> {
        self.buffers
            .get(self.current_buffer)
            .map(|b| b.title.clone())
    }

    pub fn set_command_text(&mut self, text: &str) {
        if let Some(buffer) = self.buffers.get_mut(self.current_buffer) {
            buffer.command_text = Some(text.to_owned())
        }
    }

    pub fn buffer_at(&self, idx: usize) -> Option<Rope> {
        self.buffers.get(idx).map(|b| b.text.clone())
    }

    pub fn buffer_id_at(&self, idx: usize) -> Option<Uuid> {
        self.buffers.get(idx).map(|b| b.id)
    }

    pub fn parse_command(&mut self, _config: &Config) {
        if let Some(command_text) = &self.command_text() {
            let command_str = command_text.replace(":", "");
            let command_vec = command_str.split(" ").collect::<Vec<_>>();
            match &command_vec
                .get(0)
                .map(|c| serde_plain::from_str::<Command>(c))
            {
                Some(Ok(Command::Quit)) => {
                    self.should_quit = true;
                }
                Some(Ok(Command::Write)) => {
                    let _ = self.on_save();
                    self.set_normal_mode();
                }
                Some(Ok(Command::TabNew)) => {
                    self.set_normal_mode();
                }
                Some(Ok(Command::VerticalSplit)) => {
                    if let Some(file_name) = command_vec.get(1).map(|f| f.to_string()) {
                        let buffer = if let Ok(buffer) = Buffer::new(Some(file_name)) {
                            buffer
                        } else {
                            Buffer::new(None).unwrap()
                        };
                        self.buffers.push(buffer);
                        self.windows
                            .push(Window::new((self.buffers.len() - 1) as u16));
                    } else {
                        self.windows.push(Window::new(0));
                    }
                    self.set_normal_mode();
                }
                Some(Ok(Command::Split)) => {
                    self.set_command_text("tried to split");
                    self.set_normal_mode();
                }
                Some(Err(_)) | None => {
                    self.set_command_text("Unreconginzed command");
                    self.set_normal_mode();
                }
            }
        }
    }

    pub fn new(file_name: Option<String>) -> Result<App, std::io::Error> {
        match Buffer::new(file_name) {
            Ok(buffer) => {
                let ps = SyntaxSet::load_defaults_newlines();
                let ts = ThemeSet::load_defaults();
                let syntax = ps.find_syntax_by_extension("rs").clone();
                Ok(Self {
                    //current_tab: 0,
                    //tabs: None,
                    syntax_set: ps.clone(),
                    theme_set: ts,
                    syntax: syntax.clone().unwrap().to_owned(),
                    current_window: 0,
                    current_buffer: 0,
                    buffers: vec![buffer],
                    windows: vec![Window::new(0)],
                    should_quit: false,
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn set_normal_mode(&mut self) {
        if let Some(buffer) = self.buffers.get_mut(self.current_buffer) {
            buffer.mode = Mode::Normal;
            self.set_command_text("");
        }
    }

    pub fn on_event(&mut self, event: Event<Key>, config: &Config) {
        /*
        if let Some(buffer) = self.buffers.get_mut(self.current_buffer) {
            if let Event::Input(Key::Char('\n')) = event {
                if buffer.mode == Mode::Command {
                    self.parse_command(config);
                }
            } else {
                buffer.on_event(event, config);
                self.should_quit = buffer.should_quit;
            }
        }
        */
    }

    pub fn handle_token(token: Token) -> AnyhowResult<Vec<Token>> {
        unimplemented!()
    }
}
