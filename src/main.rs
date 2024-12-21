#![allow(dead_code, unused_imports)]
use std::io::stdin;
use std::path::PathBuf;
use std::sync::mpsc;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedStreamConfig};
use log::{error, LevelFilter};
use ratatui::crossterm::terminal;
use ratatui::layout::Layout;
use ratatui::widgets::Tabs;
use ratatui_explorer::FileExplorer;
use rmusic::database::Library;
use rmusic::playback_loop::playback_loop;
use simplelog::TermLogger;

use cli::Cli;
use rmusic::playback::{PlaybackAction, PlaybackDaemon};

use anyhow::Result;
use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal,
};

mod cli;
mod ui;

macro_rules! exit_on_error {
    ($expr:expr) => {
        match $expr {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(err) => {
                error!("Exiting because of {}", err);
                std::process::exit(1);
            }
        }
    };
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut log_config = simplelog::ConfigBuilder::new();
    let mut _quiet = false;
    TermLogger::init(
        match cli.loglevel {
            0 => {
                _quiet = true;
                LevelFilter::Off
            }
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        },
        log_config
            .set_time_level(LevelFilter::Warn)
            .set_target_level(LevelFilter::Warn)
            .build(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // Audio output
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No output device available"); // Add log

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let config = supported_configs_range.next().unwrap();
    let sample_rate = if config.try_with_sample_rate(SampleRate(48000)).is_some() {
        SampleRate(48000)
    } else {
        config.max_sample_rate()
    };
    let supported_config = SupportedStreamConfig::new(
        2,
        sample_rate,
        *config.buffer_size(),
        cpal::SampleFormat::F32,
    );

    // playback Daemon
    let mut playback_daemon = PlaybackDaemon::new(sample_rate.0 as usize);
    playback_daemon.volume_level = 0.2;

    // Thread communication
    let (tx, rx) = mpsc::channel();

    // Stream setup
    let err_fn = |err| error!("an error occurred on the output audio stream: {}", err);
    let decoder = move |data: &mut [f32], callback: &_| {
        playback_loop(data, callback, &mut playback_daemon, &rx)
    };
    let stream =
        exit_on_error!(device.build_output_stream(&supported_config.into(), decoder, err_fn, None));
    exit_on_error!(stream.play());

    let ui = ui::UI::new()?;
    loop {
        terminal.draw(|frame| frame.render_widget(&ui, frame.area()))?;

        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') {
                let _ = tx.send(PlaybackAction::PlayPause);
            }
        }
    }
}
