use std::default::Default;

use anyhow::{Ok, Result};
use futures::executor::block_on;
use library_view::LibraryViewer;
use ratatui::{layout::Layout, prelude::*};
use ratatui_eventInput::Input;
use ratatui_explorer::{FileExplorer, Theme};
use rmusic::database::Library;
use rmusic_tui::settings::input::{InputMap, Navigation};
use tabs::{Artists, TabPage, TabPages};

mod library_view;
mod tabs;
mod theme;

pub struct UI<'a> {
    tab_pages: TabPages<'a>,
    library: Library,
    input_map: InputMap,
    theme: ratatui_explorer::Theme,
}

impl UI<'_> {
    pub fn new() -> Result<Self> {
        let input_map = InputMap {
            navigation: Navigation::default(),
        };

        let artist_tab = Artists::new();

        let file_exporer = FileExplorer::with_keymap((&input_map).into())?;
        // file_exporer.set_filter(vec!["opus".to_string()])?;

        let library = block_on(Library::try_new())?;

        let tab_pages = vec![
            // TabPage::Artists(artist_tab),
            TabPage::LibraryView(LibraryViewer::new(&library)?),
            TabPage::FileExplorer(file_exporer),
        ];
        let tab_pages = TabPages::new(tab_pages, &library)?;

        Ok(Self {
            tab_pages,
            library,
            input_map,
            theme: Theme::default(),
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
        // State input
        match &mut self.tab_pages.active_tab_mut() {
            TabPage::Artists(artists) => artists.handle_input(input, &self.input_map.navigation),
            TabPage::FileExplorer(file_explorer) => {
                if let Some(file) = file_explorer.handle(input)? {
                    block_on(self.library.add_file(file.path()))?;
                }
            }
            TabPage::LibraryView(library_view) => {
                library_view.handle_input(input, &self.input_map.navigation)
            }
        }
        // General input
        self.tab_pages
            .handle_input(input, &self.input_map.navigation, &self.library)?;
        Ok(())
    }
}

impl Widget for &mut UI<'_> {
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
