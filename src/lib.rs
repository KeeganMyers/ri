pub mod app;
pub mod buffer;
pub mod parser;
pub mod reflow;
pub mod token;
pub mod ui;
pub mod window;
pub mod rls;

use crate::{app::Mode, buffer::Buffer, ui::Ui, window::Window};

use anyhow::Result as AnyhowResult;
use argh::FromArgs;
use log::LevelFilter;
use log4rs::{
    append::{
        console::{ConsoleAppender, Target},
        file::FileAppender,
    },
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
    filter::threshold::ThresholdFilter,
};
extern crate log;

fn add_safe(n: u16, other: u16) -> u16{
   (n).checked_add(other).unwrap_or(n)
}

fn sub_safe(n: u16, other: u16) -> u16{
   (n).checked_sub(other).unwrap_or(n)
}

fn mut_add_safe(n: &mut u16, other: u16) {
    *n = (*n).checked_add(other).unwrap_or(*n);
}

fn mut_sub_safe(n: &mut u16, other: u16) {
    *n = (*n).checked_sub(other).unwrap_or(*n);
}


#[derive(Debug, FromArgs)]
#[argh(description = "")]
pub struct Cli {
    ///file name to open in the first tab
    #[argh(positional)]
    pub file_name: Option<String>,
}

pub fn setup_logger() -> AnyhowResult<()> {
    let level = log::LevelFilter::Info;
    let file_path = "run_log";
    let stderr = ConsoleAppender::builder().target(Target::Stderr).build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build(file_path)
        .unwrap();

    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(level)))
                .build("stderr", Box::new(stderr)),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .appender("stderr")
                .build(LevelFilter::Debug),
        )
        .unwrap();

    let _handle = log4rs::init_config(config)?;
    Ok(())
}
