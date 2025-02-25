use ratatui_eventInput::{Input, Key, Modifier, Side};

type Inputs = Vec<Input>;

pub struct InputMap {
    pub navigation: Navigation,
    pub media: Media,
}

pub struct Media {
    pub playpause: Inputs,
    pub volume_up: Inputs,
    pub volume_down: Inputs,
}

impl Default for Media {
    fn default() -> Self {
        Self {
            playpause: Input::keys(&[Key::Char('c'), Key::Char(' ')]),
            volume_up: Input::keys(&[Key::Char('+'), Key::Char('=')]),
            volume_down: Input::keys(&[Key::Char('-')]),
        }
    }
}

pub struct Navigation {
    /// Go one item up a list
    pub list_up: Inputs,
    /// Go one item down a list
    pub list_down: Inputs,
    /// Select/interact with the item selected in a list
    pub list_select: Inputs,
    /// Go back to previous list
    pub list_back: Inputs,
    /// Cancel the current action
    pub cancel: Inputs,
    /// Next tab
    pub tab_next: Inputs,
    /// Previous tab
    pub tab_previus: Inputs,
    /// (un)Hide hidden files
    pub hide_toggle: Inputs,
    /// Add an item to the queue or library depending on the context
    pub item_add: Inputs,
    /// Select an individual item, depends on the context what it does.
    pub item_set: Inputs,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            list_up: Input::keys(&[Key::Up, Key::Char('k')]),
            list_down: Input::keys(&[Key::Down, Key::Char('j')]),
            list_select: Input::keys(&[Key::Right, Key::Char('l'), Key::Enter]),
            list_back: Input::keys(&[Key::Left, Key::Char('h')]),
            cancel: Input::keys(&[Key::Esc]),
            tab_next: Input::keys(&[Key::Tab]),
            tab_previus: vec![
                Input::new_key(Key::BackTab),
                Input::new(Key::Tab, Modifier::Shift(Side::Any)),
            ],
            hide_toggle: Input::keys(&[Key::Char('H')]),
            item_add: Input::keys(&[Key::Char('a')]),
            item_set: Input::keys(&[Key::Char('p')]),
        }
    }
}
