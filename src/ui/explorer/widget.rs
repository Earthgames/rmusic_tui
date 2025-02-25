use crate::ui::Theme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Span, Text},
    widgets::{List, ListState, StatefulWidgetRef, WidgetRef},
};

use super::{File, FileExplorer};

pub struct Renderer<'a>(pub(crate) &'a FileExplorer);

impl WidgetRef for Renderer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut state = ListState::default().with_selected(Some(self.0.selected_idx()));

        let highlight_style = if self.0.current().is_dir() {
            self.0.theme().highlight_dir_style()
        } else {
            self.0.theme().highlight_item_style()
        };

        let mut list = List::new(self.0.files().iter().map(|file| file.text(self.0.theme())))
            .style(*self.0.theme().style())
            .highlight_spacing(self.0.theme().highlight_spacing().clone())
            .highlight_style(*highlight_style);

        if let Some(symbol) = self.0.theme().highlight_symbol() {
            list = list.highlight_symbol(symbol);
        }

        if let Some(block) = self.0.theme().block() {
            list = list.block(block.clone());
        }

        StatefulWidgetRef::render_ref(&list, area, buf, &mut state)
    }
}

impl File {
    /// Returns the text with the appropriate style to be displayed for the file.
    fn text(&self, theme: &Theme) -> Text<'_> {
        let style = if self.is_dir() {
            *theme.dir_style()
        } else {
            *theme.item_style()
        };
        Span::styled(self.name().to_owned(), style).into()
    }
}
