use std::{
    default::Default,
    sync::{Arc, Mutex},
};

use anyhow::{Ok, Result};
use futures::executor::block_on;
use library_view::LibraryViewer;
use log::error;
use ratatui::{layout::Layout, prelude::*};
use ratatui_eventInput::Input;
use ratatui_explorer::{FileExplorer, Theme};
use rmusic::{
    database::Library,
    playback_loop::PlaybackAction,
    queue::{Queue, QueueItem},
};
use rmusic_tui::settings::input::{InputMap, Media, Navigation};
use tabs::{input_to_log_event, QueueView, TabPage, TabPages};

mod library_view;
mod tabs;
mod theme;

pub struct UI {
    tab_pages: TabPages,
    library: Library,
    input_map: InputMap,
    theme: ratatui_explorer::Theme,
    _queue: Arc<Mutex<Queue>>,
}

impl UI {
    pub fn new(queue: Arc<Mutex<Queue>>) -> Result<Self> {
        let input_map = InputMap {
            navigation: Navigation::default(),
        };

        // let artist_tab = Artists::new();

        let file_exporer = FileExplorer::with_keymap((&input_map).into())?;
        // file_exporer.set_filter(vec!["opus".to_string()])?;

        let library = block_on(Library::try_new())?;

        let tab_pages = vec![
            // TabPage::Artists(artist_tab),
            TabPage::LibraryView(LibraryViewer::new(&library)?),
            TabPage::FileExplorer(file_exporer),
            TabPage::Queue(QueueView::new(queue.clone())),
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
            _queue: queue,
        })
    }

    fn layout() -> Layout {
        Layout::new(
            ratatui::layout::Direction::Vertical,
            vec![Constraint::Length(2), Constraint::Fill(1)],
        )
    }

    pub fn handle_input<I>(&mut self, input: I) -> Result<()>
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
                if let Some(file) = file_explorer.handle(input)? {
                    block_on(self.library.add_file(file.path()))?;
                }
            }
            TabPage::LibraryView(library_view) => {
                match library_view.handle_input(input, navigation, &self.library)? {
                    library_view::Action::Play(queue_item) => {
                        if let QueueItem::Track(track, _) = queue_item {
                            playback_action =
                                Some(PlaybackAction::Play(QueueItem::Track(track, false)));
                        }
                    }
                    library_view::Action::None => (),
                }
            }
            TabPage::TuiLogger(tui_widget_state) => {
                if let Some(event) = input_to_log_event(input, navigation) {
                    tui_widget_state.transition(event);
                }
            }
        }
        // General input
        self.tab_pages
            .handle_input(input, &self.input_map.navigation, &self.library)?;
        Ok(())
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
    }
}
