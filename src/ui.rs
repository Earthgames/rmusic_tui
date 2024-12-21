use anyhow::{Ok, Result};
use futures::executor::block_on;
use ratatui::{
    layout::Layout,
    prelude::*,
    widgets::{List, StatefulWidget, Tabs},
};
use ratatui_explorer::FileExplorer;
use rmusic::database::{self, Library};

enum State {
    Artist,
    FileExporer,
}

pub struct UI<'a> {
    state: State,
    tabs: Tabs<'a>,
    main_layout: Layout,
    file_exporer: FileExplorer,
    library: Library,
    list: List<'a>,
}

impl<'a> UI<'a> {
    pub fn new() -> Result<Self> {
        let state = State::Artist;
        let tabs = Tabs::new(vec!["Artist", "Files"]);
        let file_exporer = FileExplorer::with_filter(vec!["opus".to_string()])?;
        let main_layout = Layout::new(
            ratatui::layout::Direction::Vertical,
            vec![Constraint::Length(2), Constraint::Fill(1)],
        );
        let library = block_on(database::Library::try_new())?;
        let list = List::new(Vec::<String>::new());
        let mut result = Self {
            state,
            tabs,
            main_layout,
            file_exporer,
            library,
            list,
        };
        result.sync_with_database()?;
        Ok(result)
    }

    pub fn sync_with_database(&mut self) -> Result<()> {
        let artists = block_on(self.library.artists())?
            .into_iter()
            .map(|x| x.name)
            .collect::<Vec<_>>();
        self.list = List::new(artists);
        Ok(())
    }
}

impl<'a> Widget for &UI<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rects = self.main_layout.split(area);
        Widget::render(&self.tabs, rects[0], buf);
        let mainrect = rects[1];
        match self.state {
            State::Artist => Widget::render(&self.list, mainrect, buf),
            State::FileExporer => self.file_exporer.widget().render(mainrect, buf),
        }
    }
}
