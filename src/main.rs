#![allow(dead_code, unused_imports)]
use std::io::stdin;
use std::path::PathBuf;
use std::sync::mpsc;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedStreamConfig};
use log::{error, LevelFilter};
use ratatui::crossterm::terminal;
use ratatui_explorer::FileExplorer;
use rmusic::database::Library;
use rmusic::playback_loop::playback_loop;
use simplelog::TermLogger;

use cli::Cli;
use rmusic::playback::{PlaybackAction, PlaybackDaemon};

use std::io;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal,
};

mod cli;

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

#[tokio::main]
async fn main() -> io::Result<()> {
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

    //Database
    // let database = Library::try_new().await.expect("No database");
    //
    // if cli.add_path {
    //     database
    //         .add_file(&PathBuf::from(&cli.opus_file))
    //         .await
    //         .expect("database problem");
    // }

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(terminal);
    ratatui::restore();
    app_result
    // let mut command = "".to_string();
    // let stdin = stdin();
    // loop {
    //     command.clear();
    //     exit_on_error!(stdin.read_line(&mut command)); // Ignore all errors for now
    //     let args: Vec<&str> = command.split_ascii_whitespace().collect();
    //     match args[0] {
    //         "q" => break,
    //         "p" => exit_on_error!(tx.send(PlaybackAction::Playing)),
    //         "s" => exit_on_error!(tx.send(PlaybackAction::Paused)),
    //         "f" => exit_on_error!(tx.send(PlaybackAction::FastForward(5))),
    //         "r" => exit_on_error!(tx.send(PlaybackAction::Rewind(5))),
    //         "g" => {
    //             if args.len() < 2 {
    //                 continue;
    //             }
    //             let num = exit_on_error!(args[1].parse::<u64>());
    //             exit_on_error!(tx.send(PlaybackAction::GoTo(num)))
    //         }
    //         _ => continue,
    //     }
    // }
    // Ok(())
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
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
    playback_daemon.volume_level = 0.1;

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

    let mut file_exporer = FileExplorer::with_filter(vec!["opus".to_string()])?;
    loop {
        terminal.draw(|frame| frame.render_widget(&file_exporer.widget(), frame.area()))?;

        let event = event::read()?;
        if let event::Event::Key(key) = event {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char(' ') {
                let _ = tx.send(PlaybackAction::PlayPause);
            }
        }
        // PlaybackActions
        if let Some(music_file) = file_exporer.handle(&event)? {
            let _ = tx.send(PlaybackAction::Play(music_file.path().to_owned()));
        };
    }
}
