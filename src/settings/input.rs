use ratatui_eventInput::{Input, Key, Modifier, Side};

type Inputs = Vec<Input>;

pub struct InputMap {
    pub navigation: Navigation,
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
        }
    }
}

impl From<&InputMap> for ratatui_explorer::KeyMap {
    fn from(value: &InputMap) -> Self {
        ratatui_explorer::KeyMap {
            list_up: value.navigation.list_up.clone(),
            list_down: value.navigation.list_down.clone(),
            folder_enter: value.navigation.list_select.clone(),
            folder_exit: value.navigation.list_back.clone(),
            hide_toggle: Default::default(), // TODO: change later
        }
    }
}
