use std::{cmp::min, time::Duration};

use anyhow::Result;
use futures::executor::block_on;
use ratatui::{
    prelude::*,
    widgets::{Cell, Row, Table, TableState},
};
use ratatui_eventInput::Input;
use rmusic::{
    database::{
        artist,
        context::TrackResult,
        library_view::{LibraryView, L1, L2, L3},
        release, track, Library,
    },
    queue::QueueItem,
};
use rmusic_tui::settings::input::Navigation;
use sea_orm::ModelTrait;

use super::theme::Theme;

pub struct LibraryViewer<A, B, C>
where
    A: L1<B, C> + Sync,
    B: L2<C> + ModelTrait + Sync,
    C: L3,
    <<C as sea_orm::ModelTrait>::Entity as sea_orm::EntityTrait>::Model:
        rmusic::database::library_view::IntoFR<C>,
    <B as sea_orm::ModelTrait>::Entity: sea_orm::Related<<C as sea_orm::ModelTrait>::Entity>,
{
    table_state_l1: TableState,
    table_state_l2: TableState,
    table_state_l3: TableState,
    active_list: ActiveList,
    library_view: LibraryView<A, B, C>,
}

#[derive(PartialEq)]
enum ActiveList {
    Level1,
    Level2,
    Level3,
}

#[derive(PartialEq)]
pub enum Action {
    Play(QueueItem),
    // Add to queue,
    // Add to playlist,
    None,
}

impl From<()> for Action {
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<QueueItem> for Action {
    fn from(item: QueueItem) -> Self {
        Self::Play(item)
    }
}

impl<A, B, C> LibraryViewer<A, B, C>
where
    A: L1<B, C> + Sync,
    B: L2<C> + ModelTrait + Sync,
    C: L3,
    <<C as sea_orm::ModelTrait>::Entity as sea_orm::EntityTrait>::Model:
        rmusic::database::library_view::IntoFR<C>,
    <B as sea_orm::ModelTrait>::Entity: sea_orm::Related<<C as sea_orm::ModelTrait>::Entity>,
{
    pub fn new(library: &Library) -> Result<Self> {
        let mut table_state_l1 = TableState::default();
        table_state_l1.select(Some(0));
        let mut library_view = block_on(LibraryView::new(library))?;
        block_on(library_view.sync_with_database_all(library))?;
        Ok(LibraryViewer {
            table_state_l1,
            table_state_l2: TableState::default(),
            table_state_l3: TableState::default(),
            library_view,
            active_list: ActiveList::Level1,
        })
    }

    pub fn handle_input<I>(
        &mut self,
        input: I,
        input_map: &Navigation,
        library: &Library,
    ) -> Result<Action>
    where
        I: Into<Input>,
    {
        let input: Input = input.into();

        let mut action: Action = Action::None;

        if input_map.list_down.contains(&input) {
            self.scroll_down();
        } else if input_map.list_up.contains(&input) {
            self.scroll_up();
        } else if input_map.list_select.contains(&input) {
            if self.active_list == ActiveList::Level3 {
                let index = self.index_l3();
                action = block_on(self.library_view.get_context_list_l3(library, index))?.into()
            } else {
                self.active_list = self.next_list_state();
            }
        } else if input_map.list_back.contains(&input) {
            self.active_list = self.previous_list_state();
        } else if input_map.item_set.contains(&input) {
            action = self.get_context(library)?.into();
        } else if input_map.refresh.contains(&input) {
            block_on(self.library_view.sync_with_database_all(library))?;
        }

        self.sync_with_database(library)?;

        Ok(action)
    }

    fn scroll_up(&mut self) {
        let current_index = self.active_list_state().selected().unwrap_or(0);
        let target_index = if current_index == 0 {
            self.active_list_size() - 1
        } else {
            current_index - 1
        };
        *self.active_list_state().selected_mut() = Some(target_index);
    }

    fn scroll_down(&mut self) {
        let current_index = self.active_list_state().selected().unwrap_or(0);
        let target_index = if current_index >= self.active_list_size() - 1 {
            0
        } else {
            current_index + 1
        };
        *self.active_list_state().selected_mut() = Some(target_index);
    }

    fn active_list_size(&self) -> usize {
        let l1_index = self.table_state_l1.selected().unwrap_or(0);
        let l2_index = self.table_state_l2.selected().unwrap_or(0);
        match self.active_list {
            ActiveList::Level1 => self.library_view.get_l1().len(),
            ActiveList::Level2 => self.library_view.get_l2(l1_index).len(),
            ActiveList::Level3 => self.library_view.get_l3((l1_index, l2_index)).len(),
        }
    }

    fn get_context(&mut self, library: &Library) -> TrackResult<QueueItem> {
        match self.active_list {
            ActiveList::Level1 => {
                let index = self.index_l1();
                block_on(self.library_view.get_context_l1(library, index))
            }
            ActiveList::Level2 => {
                let index = self.index_l2();
                block_on(self.library_view.get_context_l2(library, index))
            }
            ActiveList::Level3 => {
                let index = self.index_l3();
                block_on(self.library_view.get_context_l3(library, index))
            }
        }
    }

    fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        let l1 = self.library_view.get_l1();
        if l1.is_empty() {
            return Ok(());
        }
        let ind_l1 = if let Some(ind_l1) = self.table_state_l1.selected() {
            // make sure the index exists
            min(ind_l1, l1.len() - 1)
        } else {
            // if we have not selected anything we select something for the user
            self.table_state_l1.select(Some(0));
            0
        };

        block_on(
            self.library_view
                .sync_with_database_l2_item(library, ind_l1),
        )?;

        let l2 = self.library_view.get_l2(ind_l1);
        if l2.is_empty() {
            self.active_list = ActiveList::Level1;
            return Ok(());
        }

        let ind_l2 = if let Some(ind_l2) = self.table_state_l2.selected() {
            min(ind_l2, l2.len() - 1)
        } else {
            // user will select something
            self.table_state_l2.select(Some(0));
            0
        };

        block_on(
            self.library_view
                .sync_with_database_l3_item(library, (ind_l1, ind_l2)),
        )?;

        let l3 = self.library_view.get_l3((ind_l1, ind_l2));
        if !l3.is_empty() && self.table_state_l3.selected().is_none() {
            self.table_state_l3.select(Some(0));
        } else if l3.is_empty() && self.active_list == ActiveList::Level3 {
            self.active_list = ActiveList::Level2;
        }
        Ok(())
    }

    fn index_l1(&mut self) -> usize {
        match self.table_state_l1.selected() {
            Some(index) => index,
            None => {
                // select 0 if nothing is selected
                self.table_state_l1.select(Some(0));
                0
            }
        }
    }

    fn index_l2(&mut self) -> (usize, usize) {
        match self.table_state_l2.selected() {
            Some(index) => (self.index_l1(), index),
            None => {
                // select 0 if nothing is selected
                self.table_state_l2.select(Some(0));
                (self.index_l1(), 0)
            }
        }
    }

    fn index_l3(&mut self) -> (usize, usize, usize) {
        match self.table_state_l3.selected() {
            Some(index) => {
                let l2 = self.index_l2();
                (l2.0, l2.1, index)
            }
            None => {
                // select 0 if nothing is selected
                self.table_state_l3.select(Some(0));
                let l2 = self.index_l2();
                (l2.0, l2.1, 0)
            }
        }
    }

    fn active_list_state(&mut self) -> &mut TableState {
        match self.active_list {
            ActiveList::Level1 => &mut self.table_state_l1,
            ActiveList::Level2 => &mut self.table_state_l2,
            ActiveList::Level3 => &mut self.table_state_l3,
        }
    }

    // fn active_list_state(&self) -> &TableState {
    //     match self.active_list {
    //         ActiveList::Level1 => &self.table_state_l1,
    //         ActiveList::Level2 => &self.table_state_l2,
    //         ActiveList::Level3 => &self.table_state_l3,
    //     }
    // }

    fn next_list_state(&self) -> ActiveList {
        match self.active_list {
            ActiveList::Level1 => ActiveList::Level2,
            ActiveList::Level2 => ActiveList::Level3,
            ActiveList::Level3 => ActiveList::Level3,
        }
    }

    fn previous_list_state(&self) -> ActiveList {
        match self.active_list {
            ActiveList::Level1 => ActiveList::Level1,
            ActiveList::Level2 => ActiveList::Level1,
            ActiveList::Level3 => ActiveList::Level2,
        }
    }

    fn layout() -> Layout {
        Layout::new(
            ratatui::layout::Direction::Horizontal,
            vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ],
        )
    }

    fn style<'a>(table: Table<'a>, theme: &'a Theme) -> Table<'a> {
        let mut table = table
            .style(*theme.style())
            .highlight_spacing(theme.highlight_spacing().clone())
            // .highlight_style(*theme.highlight_item_style())
            .highlight_symbol(theme.highlight_symbol().unwrap_or_default())
        // TODO: make option of padding
        // TODO: Hope they add padding for tables, (Or fix it yourself)
        // .scroll_padding(3)
        ;

        if let Some(block) = theme.block() {
            table = table.block(block.clone());
        }
        table
    }
}

pub trait Viewable {
    fn to_view(&self) -> Row;
    fn contrains() -> Vec<Constraint>;
}

fn cell_al(content: &str, alignment: Alignment) -> Cell {
    Cell::new(Text::from(content).alignment(alignment))
}

fn cell_al_string(content: String, alignment: Alignment) -> Cell<'static> {
    Cell::new(Text::from(content).alignment(alignment))
}

impl Viewable for artist::Model {
    fn to_view(&self) -> Row {
        Row::new(vec![cell_al(&self.name, Alignment::Left)])
    }
    fn contrains() -> Vec<Constraint> {
        vec![Constraint::Fill(1)]
    }
}

impl Viewable for release::Model {
    fn to_view(&self) -> Row {
        Row::new(vec![cell_al(&self.name, Alignment::Left)])
    }
    fn contrains() -> Vec<Constraint> {
        vec![Constraint::Fill(1)]
    }
}

impl Viewable for track::Model {
    fn to_view(&self) -> Row {
        let duration = Duration::from_secs(self.duration.try_into().expect("fuck the database"));
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        Row::new(vec![
            cell_al(&self.name, Alignment::Left),
            cell_al_string(format!("{:0>2}:{:0>2}", minutes, seconds), Alignment::Right),
        ])
    }
    fn contrains() -> Vec<Constraint> {
        vec![Constraint::Percentage(80), Constraint::Percentage(20)]
    }
}

impl<A, B, C> LibraryViewer<A, B, C>
where
    A: L1<B, C> + Sync + Viewable,
    B: L2<C> + ModelTrait + Sync + Viewable,
    C: L3 + Viewable,
    <<C as sea_orm::ModelTrait>::Entity as sea_orm::EntityTrait>::Model:
        rmusic::database::library_view::IntoFR<C>,
    <B as sea_orm::ModelTrait>::Entity: sea_orm::Related<<C as sea_orm::ModelTrait>::Entity>,
{
    pub fn render(&mut self, area: Rect, buffer: &mut Buffer, theme: &Theme) {
        let rects = Self::layout().split(area);
        let mut l1 = Self::style(
            Table::new(
                self.library_view.get_l1().iter().map(|x| x.to_view()),
                A::contrains(),
            ),
            theme,
        );
        if self.active_list == ActiveList::Level1 {
            l1 = l1.row_highlight_style(*theme.highlight_item_style());
        }
        StatefulWidget::render(l1, rects[0], buffer, &mut self.table_state_l1);
        if let Some(i) = self.table_state_l1.selected() {
            let mut l2 = Self::style(
                Table::new(
                    self.library_view.get_l2(i).iter().map(|x| x.to_view()),
                    B::contrains(),
                ),
                theme,
            );
            if self.active_list == ActiveList::Level2 {
                l2 = l2.row_highlight_style(*theme.highlight_item_style());
            }
            StatefulWidget::render(l2, rects[1], buffer, &mut self.table_state_l2);
            if let Some(y) = self.table_state_l2.selected() {
                let mut l3 = Self::style(
                    Table::new(
                        self.library_view.get_l3((i, y)).iter().map(|x| x.to_view()),
                        C::contrains(),
                    ),
                    theme,
                );
                if self.active_list == ActiveList::Level3 {
                    l3 = l3.row_highlight_style(*theme.highlight_item_style());
                }
                StatefulWidget::render(l3, rects[2], buffer, &mut self.table_state_l3)
            }
        }
    }
}
