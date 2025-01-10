use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, SupportedStreamConfig};
use log::error;
use rmusic::playback_loop::playback_loop;

use rmusic::playback::{PlaybackAction, PlaybackDaemon};

use anyhow::Result;

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
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
    let mut _quiet = false;

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

    let mut ui = ui::UI::new()?;
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
        ui.handle_input(&event)?;
    }
}
