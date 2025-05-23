use anyhow::{Ok, Result};
use futures::executor::block_on;
use ratatui::widgets::Widget;
use ratatui::{
    prelude::*,
    widgets::{List, ListState, Tabs},
};
use ratatui_eventInput::{Input, Key};
use rmusic::database::{self, artist, release, track, Library};
use rmusic::playback::playback_context::ArcPlaybackContext;
use rmusic::queue::QueueItem;
use rmusic_tui::settings::input::Navigation;
use tui_logger::*;

use super::library_view::LibraryViewer;
use super::theme::Theme;
use super::FileExplorer;

pub struct TabPages {
    tab_pages: Vec<TabPage>,
    active_tab_index: usize,
}

#[allow(dead_code)]
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

    pub fn add_tab(&mut self, tab_page: TabPage) {
        self.tab_pages.push(tab_page);
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

#[allow(dead_code)]
pub enum TabPage {
    Artists(Artists),
    FileExplorer(FileExplorer),
    LibraryView(LibraryViewer<artist::Model, release::Model, track::Model>),
    TuiLogger(TuiWidgetState),
    Queue(QueueView),
}

impl TabPage {
    pub fn tab_name(&self) -> &'static str {
        match self {
            TabPage::Artists(_) => "Artist",
            TabPage::FileExplorer(_) => "Files",
            TabPage::LibraryView(_) => "LibraryView",
            TabPage::TuiLogger(_) => "TuiLogger",
            TabPage::Queue(_) => "Queue",
        }
    }
    pub fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        match self {
            TabPage::Artists(artists) => artists.sync_with_database(library),
            _ => Ok(()),
        }
    }
    pub fn render(
        &mut self,
        rect: Rect,
        buffer: &mut Buffer,
        theme: &Theme,
        playback_context: &ArcPlaybackContext,
    ) {
        match self {
            TabPage::Artists(artists) => artists.render(rect, buffer, theme),
            TabPage::FileExplorer(file_explorer) => file_explorer.widget().render(rect, buffer),
            TabPage::LibraryView(library_viewer) => library_viewer.render(rect, buffer, theme),
            TabPage::TuiLogger(tui_widget_state) => TuiLoggerSmartWidget::default()
                .style_error(Style::default().fg(Color::Red))
                .style_debug(Style::default().fg(Color::Green))
                .style_warn(Style::default().fg(Color::Yellow))
                .style_trace(Style::default().fg(Color::Magenta))
                .style_info(Style::default().fg(Color::Cyan))
                .output_separator(':')
                .output_timestamp(Some("%H:%M:%S%.f6".to_string()))
                .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
                .output_target(true)
                .output_file(true)
                .output_line(true)
                .state(tui_widget_state)
                .render(rect, buffer),
            TabPage::Queue(queue) => queue.render(rect, buffer, playback_context),
        }
    }
}

pub struct QueueView {
    list_state: ListState,
}

impl QueueView {
    pub fn new() -> QueueView {
        QueueView {
            list_state: ListState::default(),
        }
    }

    pub fn render(
        &mut self,
        rect: Rect,
        buffer: &mut Buffer,
        playback_context: &ArcPlaybackContext,
    ) {
        let queue = playback_context.lock_queue();
        let list = List::new(queue.queue_items().iter().map(show_queue_item));
        StatefulWidget::render(list, rect, buffer, &mut self.list_state);
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
}

fn show_queue_item(item: &QueueItem) -> &str {
    match item {
        QueueItem::Track(path_buf) => path_buf.to_str().unwrap(),
        QueueItem::PlayList(_, _) => "Playlist",
        QueueItem::Album(_, _) => "Album",
    }
}

#[derive(Clone)]
pub struct Artists {
    list_state: ListState,
    list: Vec<database::Artist>,
}

#[allow(dead_code)]
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

    pub fn render(&mut self, rect: Rect, buffer: &mut Buffer, theme: &Theme) {
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

pub fn input_to_log_event<I>(input: I, input_map: &Navigation) -> Option<TuiWidgetEvent>
where
    I: Into<Input>,
{
    let input: Input = input.into();
    if input_map.list_down.contains(&input) {
        Some(TuiWidgetEvent::DownKey)
    } else if input_map.list_up.contains(&input) {
        Some(TuiWidgetEvent::UpKey)
    } else if input_map.list_back.contains(&input) {
        Some(TuiWidgetEvent::LeftKey)
    } else if input_map.list_select.contains(&input) {
        Some(TuiWidgetEvent::RightKey)
    } else if input == Input::new_key(Key::Char('h')) {
        Some(TuiWidgetEvent::HideKey)
    } else if input == Input::new_key(Key::Char('f')) {
        Some(TuiWidgetEvent::FocusKey)
    } else if input == Input::new_key(Key::PageUp) {
        Some(TuiWidgetEvent::PrevPageKey)
    } else if input == Input::new_key(Key::PageDown) {
        Some(TuiWidgetEvent::NextPageKey)
    } else {
        None
    }
}
