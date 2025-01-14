use anyhow::{Ok, Result};
use futures::executor::block_on;
use ratatui::{
    prelude::*,
    widgets::{List, ListState, Tabs},
};
use ratatui_eventInput::Input;
use ratatui_explorer::FileExplorer;
use rmusic::database::{self, artist, Library};
use rmusic_tui::settings::input::Navigation;

pub struct TabPages {
    tab_pages: Vec<TabPage>,
    active_tab_index: usize,
}

impl TabPages {
    pub fn new(tab_pages: Vec<TabPage>, library: &Library) -> Result<TabPages> {
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

    pub fn active_tab_mut(&mut self) -> &mut TabPage {
        &mut self.tab_pages[self.active_tab_index]
    }

    pub fn active_tab(&self) -> &TabPage {
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

pub enum TabPage {
    Artists(Artists),
    FileExplorer(FileExplorer),
}

impl TabPage {
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
    pub fn render(&mut self, rect: Rect, buffer: &mut Buffer, theme: &ratatui_explorer::Theme) {
        match self {
            TabPage::Artists(artists) => artists.render(rect, buffer, theme),
            TabPage::FileExplorer(file_explorer) => file_explorer.widget().render(rect, buffer),
        }
    }
}

#[derive(Clone)]
pub struct Artists {
    list_state: ListState,
    list: Vec<database::Artist>,
}

impl Artists {
    pub fn new() -> Artists {
        Artists {
            list_state: ListState::default(),
            list: vec![],
        }
    }

    pub fn handle_input<I>(&mut self, input: I, input_map: &Navigation)
    where
        I: Into<Input>,
    {
        let input: Input = input.into();
        if input_map.list_down.contains(&input) {
            self.list_state.scroll_down_by(1);
        } else if input_map.list_up.contains(&input) {
            self.list_state.scroll_up_by(1);
        }
    }

    pub fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        self.list = block_on(library.find_all::<artist::Entity>())?
            .into_iter()
            .collect::<Vec<_>>();
        Ok(())
    }

    pub fn render(&mut self, rect: Rect, buffer: &mut Buffer, theme: &ratatui_explorer::Theme) {
        let highlight_style = theme.highlight_item_style();

        let mut widget_list = List::new(self.list.iter().map(|x| x.name.as_str()))
            .style(*theme.style())
            .highlight_spacing(theme.highlight_spacing().clone())
            .highlight_style(*highlight_style)
            .highlight_symbol(theme.highlight_symbol().unwrap_or_default())
            // TODO: make option of padding
            .scroll_padding(3);

        if let Some(block) = theme.block() {
            widget_list = widget_list.block(block.clone());
        }
        StatefulWidget::render(widget_list, rect, buffer, &mut self.list_state)
    }
}
