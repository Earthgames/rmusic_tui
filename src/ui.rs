use std::{
    default::Default,
    f64,
    sync::{atomic::AtomicU8, Arc},
    thread,
};

use anyhow::{Ok, Result};
use explorer::FileExplorer;
use futures::executor::block_on;
use library_view::LibraryViewer;
use log::error;
use ratatui::{layout::Layout, prelude::*, widgets::LineGauge};
use ratatui_eventInput::Input;
use rmusic::{
    database::Library, playback::playback_context::ArcPlaybackContext,
    playback_loop::PlaybackAction, queue::Queue,
};
use rmusic_tui::settings::input::{InputMap, Media, Navigation};
use tabs::{input_to_log_event, QueueView, TabPage, TabPages};
use theme::Theme;

mod explorer;
mod library_view;
mod tabs;
mod theme;

pub struct UI {
    tab_pages: TabPages,
    library: Library,
    input_map: InputMap,
    theme: Theme,
    playback_context: ArcPlaybackContext,
}

impl UI {
    pub fn new(playback_context: ArcPlaybackContext) -> Result<Self> {
        let input_map = InputMap {
            navigation: Navigation::default(),
            media: Media::default(),
        };

        // let artist_tab = Artists::new();

        let file_exporer = FileExplorer::new()?;
        // file_exporer.set_filter(vec!["opus".to_string()])?;

        let library = block_on(Library::try_new())?;

        let tab_pages = vec![
            // TabPage::Artists(artist_tab),
            TabPage::LibraryView(LibraryViewer::new(&library)?),
            TabPage::FileExplorer(file_exporer),
            // TabPage::Queue(QueueView::new(queue.clone())),
            TabPage::TuiLogger(
                tui_logger::TuiWidgetState::new().set_default_display_level(log::LevelFilter::Warn),
            ),
        ];
        let tab_pages = TabPages::new(tab_pages, &library)?;

        Ok(Self {
            tab_pages,
            library,
            input_map,
            theme: Theme::default(),
            playback_context,
        })
    }

    fn layout() -> Layout {
        Layout::new(
            ratatui::layout::Direction::Vertical,
            vec![
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(1),
            ],
        )
    }

    pub fn handle_input<I>(&mut self, input: I) -> Result<Option<PlaybackAction>>
    where
        I: Into<Input>,
    {
        let input: Input = input.into();
        let mut playback_action: Option<PlaybackAction> = None;
        let navigation = &self.input_map.navigation;
        // State input
        match &mut self.tab_pages.active_tab_mut() {
            TabPage::Artists(artists) => artists.handle_input(input, navigation),
            TabPage::FileExplorer(file_explorer) => {
                if let Some(file) = file_explorer.handle(input, navigation)? {
                    if file.is_dir() {
                        let progress = Arc::new(AtomicU8::new(0));
                        let db = self.library.clone();
                        let path = file.path().to_path_buf();
                        thread::spawn(move || {
                            if let Err(err) = block_on(db.add_folder_rec(&path, &progress)) {
                                error!("Error while adding folder to library: {:?}", err);
                            }
                        });
                    } else if let Err(err) = block_on(self.library.add_file(file.path())) {
                        error!("Error while adding file to library: {:?}", err);
                    }
                }
            }
            TabPage::LibraryView(library_view) => {
                match library_view.handle_input(input, navigation, &self.library)? {
                    library_view::Action::Play(queue_item) => {
                        playback_action = Some(PlaybackAction::Play(queue_item));
                    }
                    library_view::Action::None => (),
                }
            }
            TabPage::TuiLogger(tui_widget_state) => {
                if let Some(event) = input_to_log_event(input, navigation) {
                    tui_widget_state.transition(event);
                }
            }
            TabPage::Queue(queue_view) => queue_view.handle_input(input, navigation),
        }
        if playback_action.is_some() {
            return Ok(playback_action);
        }
        // General input
        let media = &self.input_map.media;
        if media.playpause.contains(&input) {
            playback_action = Some(PlaybackAction::PlayPause);
        } else if media.volume_up.contains(&input) {
            playback_action = Some(PlaybackAction::ChangeVolume(0.02))
        } else if media.volume_down.contains(&input) {
            playback_action = Some(PlaybackAction::ChangeVolume(-0.02))
        }

        if playback_action.is_some() {
            return Ok(playback_action);
        }

        self.tab_pages
            .handle_input(input, &self.input_map.navigation, &self.library)?;
        Ok(playback_action)
    }
}

impl Widget for &mut UI {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let rects = UI::layout().split(area);
        self.tab_pages.widget().render(rects[0], buf);
        let mainrect = rects[1];
        self.tab_pages
            .active_tab_mut()
            .render(mainrect, buf, &self.theme);
        let length = self.playback_context.length();
        let left = self.playback_context.left();
        let played = length - left;
        LineGauge::default()
            .ratio(if played == 0 {
                0.0
            } else {
                played as f64 / length as f64
            })
            .filled_style(Style::new().white().bold())
            .unfilled_style(Style::new().black())
            //TODO: CHANGE this with `unfilled_char()` when going to 0.30
            .line_set(symbols::line::THICK)
            .render(rects[2], buf);
    }
}
