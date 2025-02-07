use anyhow::Result;
use futures::executor::block_on;
use ratatui::{
    prelude::*,
    widgets::{Cell, Row, Table, TableState},
};
use ratatui_eventInput::Input;
use rmusic::database::{
    artist,
    library_view::{LibraryView, L1, L2, L3},
    Library,
};
use rmusic_tui::settings::input::Navigation;
use sea_orm::ModelTrait;

pub struct LibraryViewer<'a, A, B, C>
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
    _lf: std::marker::PhantomData<&'a str>,
}

enum ActiveList {
    Level1,
    Level2,
    Level3,
}

impl<A, B, C> LibraryViewer<'_, A, B, C>
where
    A: L1<B, C> + Sync,
    B: L2<C> + ModelTrait + Sync,
    C: L3,
    <<C as sea_orm::ModelTrait>::Entity as sea_orm::EntityTrait>::Model:
        rmusic::database::library_view::IntoFR<C>,
    <B as sea_orm::ModelTrait>::Entity: sea_orm::Related<<C as sea_orm::ModelTrait>::Entity>,
{
    pub fn new(library: &Library) -> Result<Self> {
        Ok(LibraryViewer {
            table_state_l1: TableState::default(),
            table_state_l2: TableState::default(),
            table_state_l3: TableState::default(),
            library_view: block_on(LibraryView::new(library))?,
            active_list: ActiveList::Level1,
            _lf: std::marker::PhantomData,
        })
    }

    pub fn handle_input<I>(&mut self, input: I, input_map: &Navigation)
    where
        I: Into<Input>,
    {
        let active_table_state = self.active_list_state();
        let input: Input = input.into();
        if input_map.list_down.contains(&input) {
            active_table_state.scroll_down_by(1);
        } else if input_map.list_up.contains(&input) {
            active_table_state.scroll_up_by(1);
        }
    }

    pub fn active_list_state(&mut self) -> &mut TableState {
        match self.active_list {
            ActiveList::Level1 => &mut self.table_state_l1,
            ActiveList::Level2 => &mut self.table_state_l2,
            ActiveList::Level3 => &mut self.table_state_l3,
        }
    }

    pub fn sync_with_database(&mut self, library: &Library) -> Result<()> {
        Ok(block_on(self.library_view.sync_with_database_l1(library))?)
    }
}

pub trait Viewable {
    fn to_view(&self) -> Row;
    fn contrains() -> Vec<Constraint>;
}

fn cell_al(content: &str, alignment: Alignment) -> Cell {
    Cell::new(Text::from(content).alignment(alignment))
}

impl Viewable for artist::Model {
    fn to_view(&self) -> Row {
        Row::new(vec![
            cell_al(&self.name, Alignment::Left),
            cell_al(&self.name, Alignment::Right),
        ])
    }
    fn contrains() -> Vec<Constraint> {
        vec![Constraint::Fill(1), Constraint::Fill(1)]
    }
}

impl<A, B, C> LibraryViewer<'_, A, B, C>
where
    A: L1<B, C> + Sync + Viewable,
    B: L2<C> + ModelTrait + Sync,
    C: L3,
    <<C as sea_orm::ModelTrait>::Entity as sea_orm::EntityTrait>::Model:
        rmusic::database::library_view::IntoFR<C>,
    <B as sea_orm::ModelTrait>::Entity: sea_orm::Related<<C as sea_orm::ModelTrait>::Entity>,
{
    pub fn render(&mut self, rect: Rect, buffer: &mut Buffer, theme: &ratatui_explorer::Theme) {
        let highlight_style = theme.highlight_item_style();

        let mut widget_list = Table::new(
            self.library_view.get_l1().iter().map(|x| x.to_view()),
            A::contrains(),
        )
        .style(*theme.style())
        .highlight_spacing(theme.highlight_spacing().clone())
        .highlight_style(*highlight_style)
        .highlight_symbol(theme.highlight_symbol().unwrap_or_default())
        // TODO: make option of padding
        // TODO: Hope they add padding for tables, (Or fix it yourself)
        // .scroll_padding(3);
        ;

        if let Some(block) = theme.block() {
            widget_list = widget_list.block(block.clone());
        }
        StatefulWidget::render(widget_list, rect, buffer, &mut self.table_state_l1)
    }
}
