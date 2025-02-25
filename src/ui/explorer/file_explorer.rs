use std::{io::Result, path::PathBuf};

use ratatui::widgets::WidgetRef;
use ratatui_eventInput::{Input, Key};
use rmusic_tui::settings::input::Navigation;

use super::widget::Renderer;
use crate::ui::Theme;

#[derive(Debug, Clone)]
pub struct FileExplorer {
    cwd: PathBuf,
    files: Vec<File>,
    selected: usize,
    theme: crate::ui::Theme,
    filter: Vec<String>,
    show_hidden: bool,
}

#[allow(dead_code)]
impl FileExplorer {
    pub fn new() -> Result<FileExplorer> {
        let cwd = std::env::current_dir()?;

        let mut file_explorer = Self {
            cwd,
            files: vec![],
            selected: 0,
            theme: Theme::default(),
            filter: vec![],
            show_hidden: false,
        };

        file_explorer.get_and_set_files()?;

        Ok(file_explorer)
    }

    #[inline]
    pub fn with_theme(theme: Theme) -> Result<FileExplorer> {
        let mut file_explorer = Self::new()?;

        file_explorer.theme = theme;

        Ok(file_explorer)
    }

    #[inline]
    pub fn with_filter(filter: Vec<String>) -> Result<FileExplorer> {
        let mut file_explorer = Self::new()?;

        file_explorer.filter = filter;
        file_explorer.get_and_set_files()?;

        Ok(file_explorer)
    }

    #[inline]
    pub const fn widget(&self) -> impl WidgetRef + '_ {
        Renderer(self)
    }

    pub fn handle<I: Into<Input>>(
        &mut self,
        input: I,
        key_map: &Navigation,
    ) -> Result<Option<&File>> {
        let input: Input = input.into();
        let last_index = self.files.len() - 1;

        if input.key == Key::Right {
            println!("right")
        } else if input.key == Key::Left {
            println!("left")
        }

        // default keys used in comments
        // Up key
        if key_map.list_up.contains(&input) {
            if self.selected == 0 {
                self.selected = last_index;
            } else {
                self.selected -= 1;
            }
        // Down key
        } else if key_map.list_down.contains(&input) {
            if self.selected == last_index {
                self.selected = 0;
            } else {
                self.selected += 1;
            }
        // Left key
        } else if key_map.list_back.contains(&input) {
            let parent = self.cwd.parent();

            if let Some(parent) = parent {
                self.cwd = parent.to_path_buf();
                self.get_and_set_files()?;
                self.selected = 0
            }
        // Right key
        } else if key_map.list_select.contains(&input) {
            if self.files[self.selected].path.is_dir() {
                self.cwd = self.files.swap_remove(self.selected).path;
                self.get_and_set_files()?;
                self.selected = 0
            } else {
                return Ok(Some(self.current()));
            }
        // `H` key
        } else if key_map.hide_toggle.contains(&input) {
            self.show_hidden = !self.show_hidden;
            self.get_and_set_files()?;
            if self.selected >= self.files.len() {
                self.selected = last_index;
            }
        }

        Ok(None)
    }

    #[inline]
    pub fn set_cwd<P: Into<PathBuf>>(&mut self, cwd: P) -> Result<()> {
        self.cwd = cwd.into();
        self.get_and_set_files()?;
        self.selected = 0;

        Ok(())
    }

    #[inline]
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_filter(&mut self, filter: Vec<String>) -> Result<()> {
        self.filter = filter;
        self.get_and_set_files()
    }

    #[inline]
    pub fn set_selected_idx(&mut self, selected: usize) {
        assert!(selected < self.files.len());
        self.selected = selected;
    }

    #[inline]
    pub fn current(&self) -> &File {
        &self.files[self.selected]
    }

    #[inline]
    pub const fn cwd(&self) -> &PathBuf {
        &self.cwd
    }

    #[inline]
    pub const fn files(&self) -> &Vec<File> {
        &self.files
    }

    #[inline]
    pub const fn selected_idx(&self) -> usize {
        self.selected
    }

    #[inline]
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    fn get_and_set_files(&mut self) -> Result<()> {
        let (mut dirs, mut none_dirs): (Vec<_>, Vec<_>) = std::fs::read_dir(&self.cwd)?
            .filter_map(|entry| {
                if let Ok(e) = entry {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = match e {
                        e if is_dir => format!("{}/", e.file_name().to_string_lossy()),
                        e if self.filter.is_empty()
                            || self.filter.contains(
                                &path
                                    .extension()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                            ) =>
                        {
                            e.file_name().to_string_lossy().into_owned()
                        }
                        _ => return None,
                    };
                    if !self.show_hidden && name.starts_with('.') {
                        None
                    } else {
                        Some(File { name, path, is_dir })
                    }
                } else {
                    None
                }
            })
            .partition(|file| file.is_dir);

        dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));
        none_dirs.sort_unstable_by(|f1, f2| f1.name.cmp(&f2.name));

        if let Some(parent) = self.cwd.parent() {
            let mut files = Vec::with_capacity(1 + dirs.len() + none_dirs.len());

            files.push(File {
                name: "../".to_owned(),
                path: parent.to_path_buf(),
                is_dir: true,
            });

            files.extend(dirs);
            files.extend(none_dirs);

            self.files = files
        } else {
            let mut files = Vec::with_capacity(dirs.len() + none_dirs.len());

            files.extend(dirs);
            files.extend(none_dirs);

            self.files = files;
        };

        Ok(())
    }
}

/// A file or directory in the file explorer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct File {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

impl File {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub const fn path(&self) -> &PathBuf {
        &self.path
    }

    #[inline]
    pub const fn is_dir(&self) -> bool {
        self.is_dir
    }
}
