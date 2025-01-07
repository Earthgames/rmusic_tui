use anyhow::{Ok, Result};
use futures::executor::block_on;
use ratatui::{prelude::*, widgets::List};
use ratatui_explorer::FileExplorer;
use rmusic::database::Library;

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
