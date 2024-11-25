use ratatui::{
    layout::Layout,
    prelude::*,
    widgets::{List, StatefulWidget, Tabs},
};
use ratatui_explorer::FileExplorer;

enum State {
    Artist,
    FileExporer,
}

pub struct UI<'a> {
    state: State,
    tabs: Tabs<'a>,
    main_layout: Layout,
    file_exporer: FileExplorer,
    list: List<'a>,
}

impl<'a> UI<'a> {
    pub fn new() -> Result<Self, std::io::Error> {
        let state = State::Artist;
        let tabs = Tabs::new(vec!["Artist", "Files"]);
        let file_exporer = FileExplorer::with_filter(vec!["opus".to_string()])?;
        let main_layout = Layout::new(ratatui::layout::Direction::Horizontal, vec![10, 90]);
        let list = List::new(Vec::<String>::new());
        Ok(Self {
            state,
            tabs,
            main_layout,
            file_exporer,
            list,
        })
    }
}

impl<'a> Widget for UI<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let rects = self.main_layout.split(area);
        self.tabs.render(rects[0], buf);
        let mainrect = rects[1];
        match self.state {
            State::Artist => Widget::render(self.list, mainrect, buf),
            State::FileExporer => self.file_exporer.widget().render(mainrect, buf),
        }
    }
}
