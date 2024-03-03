use crossterm;
use crossterm::event::{poll, read, Event};
use ri::{
    app::App,
    parser::{Parser, UserInput},
    token::{display_token::DisplayToken, Token},
};
use ri::{setup_logger, Cli,rls::embed_rls};
use std::time::Duration;

use std::error::Error;
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let _ = setup_logger();
    embed_rls().await?;
    let mut app = App::new(cli.file_name)?;
    app.handle_tokens(vec![Token::Display(DisplayToken::DrawViewPort)]);
    let mut parser = Parser::new();
    loop {
        if let Ok(true) = poll(Duration::from_millis(250)) {
            if let Ok(Event::Key(event)) = read() {
                let tokens = parser.handle_event(UserInput { event }, &app.mode);
                if !tokens.is_empty() {
                    app.handle_tokens(tokens);
                }
                if app.should_quit {
                    break;
                }
                ()
            }
        }
    }
    Ok(())
}
