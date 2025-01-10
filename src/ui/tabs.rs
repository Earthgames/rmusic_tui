use anyhow::{Ok, Result};
use futures::executor::block_on;
use ratatui::{
    prelude::*,
    widgets::{List, Tabs},
};
use ratatui_eventInput::Input;
use ratatui_explorer::FileExplorer;
use rmusic::database::Library;
use rmusic_tui::settings::input::Navigation;

pub struct TabPages<'a> {
    tab_pages: Vec<TabPage<'a>>,
    active_tab_index: usize,
}

impl<'a> TabPages<'a> {
    pub fn new(tab_pages: Vec<TabPage<'a>>, library: &Library) -> Result<TabPages<'a>> {
        let mut tab_pages = TabPages {
            tab_pages,
            active_tab_index: 0,
        };
        tab_pages.sync_with_database(library)?;
        Ok(tab_pages)
    }

    pub fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        self.tab_pages[self.active_tab_index].sync_with_database(library)
    }

    pub fn active_tab_mut(&mut self) -> &mut TabPage<'a> {
        &mut self.tab_pages[self.active_tab_index]
    }

    pub fn active_tab(&self) -> &TabPage<'a> {
        &self.tab_pages[self.active_tab_index]
    }

    pub fn handle_input<I>(
        &mut self,
        input: I,
        input_map: &Navigation,
        library: &Library,
    ) -> Result<()>
    where
        I: Into<Input>,
    {
        let input: Input = input.into();

        if input_map.tab_next.contains(&input) {
            self.active_tab_index = (self.active_tab_index + 1) % self.tab_pages.len();
            self.sync_with_database(library)?;
        } else if input_map.tab_previus.contains(&input) {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tab_pages.len() - 1
            } else {
                self.active_tab_index - 1
            };
            self.sync_with_database(library)?;
        }
        Ok(())
    }
    pub fn widget(&self) -> Tabs {
        Tabs::new(self.tab_pages.iter().map(TabPage::tab_name)).select(self.active_tab_index)
    }
}

pub enum TabPage<'a> {
    Artists(Artists<'a>),
    FileExplorer(FileExplorer),
}

impl TabPage<'_> {
    pub fn tab_name(&self) -> &'static str {
        match self {
            TabPage::Artists(_) => "Artist",
            TabPage::FileExplorer(_) => "Files",
        }
    }
    pub fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        match self {
            TabPage::Artists(artists) => artists.sync_with_database(library),
            _ => Ok(()),
        }
    }
    pub fn render(&self, rect: Rect, buffer: &mut Buffer) {
        match self {
            TabPage::Artists(artists) => artists.render(rect, buffer),
            TabPage::FileExplorer(file_explorer) => file_explorer.widget().render(rect, buffer),
        }
    }
}

#[derive(Clone)]
pub struct Artists<'a> {
    list: List<'a>,
}

impl<'a> Artists<'a> {
    pub fn new() -> Artists<'a> {
        Artists {
            list: List::new(Vec::<String>::new()),
        }
    }

    pub fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        let artists = block_on(library.artists())?
            .into_iter()
            .map(|x| x.name)
            .collect::<Vec<_>>();
        self.list = List::new(artists);
        Ok(())
    }

    pub fn render(&self, rect: Rect, buffer: &mut Buffer) {
        Widget::render(&self.list, rect, buffer)
    }
}
