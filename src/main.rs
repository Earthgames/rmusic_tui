#[cfg(not(debug_assertions))]
use std::env;
#[cfg(debug_assertions)]
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedStreamConfig};
use log::error;
use rmusic::playback_loop::playback_loop;

use rmusic::playback::PlaybackDaemon;

use anyhow::Result;

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};
use tui_logger::{
    init_logger, set_default_level, set_log_file, TuiLoggerFile, TuiLoggerLevelOutput,
};

mod cli;
mod ui;

const FRAMERATE: u64 = 144;
/// milliseconds per frame
const MILL_FPS: u64 = 1 / (FRAMERATE * 1000);

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
    let mut _quiet = false;
    init_logger(log::LevelFilter::Debug)?;
    set_default_level(log::LevelFilter::Trace);

    #[cfg(debug_assertions)]
    let mut cach_dir = PathBuf::from(".");
    #[cfg(not(debug_assertions))]
    let mut cach_dir = env::temp_dir();
    cach_dir.push("rmusic-tui.log");

    let file_options = TuiLoggerFile::new(cach_dir.to_str().unwrap())
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_file(false)
        .output_separator(':');
    set_log_file(file_options);

    let app_result = run();
    ratatui::restore();
    app_result
}

fn run() -> Result<()> {
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
    playback_daemon.set_volume(0.2);

    // Thread communication
    let (tx, rx) = mpsc::channel();

    // ui
    let mut ui = ui::UI::new(playback_daemon.get_playback_context())?;

    // Stream setup
    let err_fn = |err| error!("an error occurred on the output audio stream: {:?}", err);
    let decoder = move |data: &mut [f32], callback: &_| {
        playback_loop(data, callback, &mut playback_daemon, &rx)
    };
    let stream =
        exit_on_error!(device.build_output_stream(&supported_config.into(), decoder, err_fn, None));
    exit_on_error!(stream.play());

    let mut terminal = ratatui::init();
    terminal.clear()?;
    loop {
        terminal.draw(|frame| frame.render_widget(&mut ui, frame.area()))?;

        // Check if we have to handle input
        if event::poll(Duration::from_millis(MILL_FPS))? {
            // Handel all input in this frame, not just one
            while event::poll(Duration::from_secs(0))? {
                let event = event::read()?;
                if let event::Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        return Ok(());
                    }
                }
                if let Some(action) = ui.handle_input(&event)? {
                    let _ = tx.send(action);
                }
            }
        }
    }
}
