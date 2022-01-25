use std::fmt::Display;

use crossterm::event::{
    read, Event, KeyCode,
    KeyCode::{Down, Up},
    KeyEvent, KeyModifiers,
};

use crate::{line::Line, SelectDialogKey};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
/// Struct to create a select dialog and get the users chosen item
///
/// The input is retrieved over an endless loop. When the user presses enter,
/// the loop stops and the chosen item is returned.
///
/// # Example
///
/// Create the dialog with default settings
///
/// ```
/// let selected_item = Select::new(vec!["item1", "item2", "item3"]).start()
/// ```
///
/// Customize dialog before starting
///
/// ```
/// let selected_item = Select::new(&vec!["item1", "item2", "item3"])
///     .add_up_key(KeyCode::Char('j'))
///     .pointer('◉')
///     .not_selected_pointer('○')
///     .underline_selected_item()
///     .start();
/// ```
struct Select<'a, I>
where
    I: ToString + Display,
    // F: Fn(SelectDialogKey, &I),
{
    items: &'a Vec<I>,
    lines: Vec<Line>,
    selected_item: usize,
    pointer: char,
    not_selected_pointer: Option<char>,
    default_up: KeyCode,
    default_down: KeyCode,
    up_keys: Vec<KeyCode>,
    down_keys: Vec<KeyCode>,
    pub selection_changed: Option<Box<dyn Fn(SelectDialogKey, &I)>>,
    move_selected_item_forward: bool,
    underline_selected_item: bool,
}

impl<'a, I> Select<'a, I>
where
    I: ToString + Display + core::fmt::Debug,
    // F: Fn(SelectDialogKey, &I),
{
    pub fn new(items: &'a Vec<I>) -> Self {
        Select {
            items,
            pointer: '>',
            selected_item: 0,
            default_up: Up,
            default_down: Down,
            selection_changed: None,
            not_selected_pointer: None,
            move_selected_item_forward: false,
            underline_selected_item: false,
            up_keys: vec![],
            down_keys: vec![],
            lines: vec![],
        }
    }
    fn build_lines(&mut self) {
        self.lines = self
            .items
            .iter()
            .map(|item| Line::new(item.to_string(), self.pointer))
            .collect();
    }
    fn print_lines(&mut self) {
        self.lines.iter_mut().for_each(|line| line.default());

        self.lines[self.selected_item].select();

        if self.underline_selected_item {
            self.lines[self.selected_item].underline();
        }
        if self.move_selected_item_forward {
            self.lines[self.selected_item].space_from_pointer(1);
        }

        self.lines.iter().for_each(|line| println!("{}", line))
    }
    fn erase_printed_items(&self) {
        self.move_n_lines_up(4);

        self.items
            .into_iter()
            .for_each(|item| println!("{}", " ".repeat(item.to_string().chars().count() + 3)));

        self.move_n_lines_up(4);
    }
    fn move_n_lines_up(&self, n: u32) {
        println!("[33[{}A", n);
    }

    fn move_up(&mut self) {
        if self.selected_item == 0 {
            return;
        };
        // let selected_item = Select::new(&vec!["item1", "item2", "item3"])
        //     .add_up_key(KeyCode::Char('j'))
        //     .pointer('◉')
        //     .not_selected_pointer('○')
        //     .underline_selected_item()
        //     .start();
        self.selected_item -= 1;
        self.erase_printed_items();
        self.print_lines();
    }
    fn move_down(&mut self) {
        if self.selected_item == self.items.len() - 1 {
            return;
        }

        self.selected_item += 1;
        self.erase_printed_items();
        self.print_lines();
    }
    fn call_event_handler_if_supplied(&self, key: SelectDialogKey) {
        if let Some(event_handler) = self.selection_changed.as_ref() {
            let current_item = &self.items.to_owned()[self.selected_item];
            event_handler(key, current_item);
        }
    }
    pub fn start(&mut self) -> &I {
        self.build_lines();
        self.print_lines();

        self.up_keys.push(self.default_up);
        self.down_keys.push(self.default_down);

        loop {
            let event = read().unwrap();

            if event
                == Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                })
            {
                break;
            }
            if self.event_contains_key(event, &self.up_keys) {
                self.move_up();
                self.call_event_handler_if_supplied(SelectDialogKey::UpKey);
                continue;
            } else if self.event_contains_key(event, &self.down_keys) {
                self.move_down();
                self.call_event_handler_if_supplied(SelectDialogKey::DownKey);
                continue;
            }
        }
        &self.items.to_owned()[self.selected_item]
    }
    fn event_contains_key(&self, event: Event, keys: &Vec<KeyCode>) -> bool {
        for key in keys.iter() {
            if event
                == Event::Key(KeyEvent {
                    code: key.clone(),
                    modifiers: KeyModifiers::NONE,
                })
            {
                return true;
            }
        }
        false
    }
    /// Set a custom pointer to show in the select dialog
    pub fn pointer(&mut self, pointer: char) -> &mut Self {
        self.pointer = pointer;
        self
    }
    pub fn set_up_key(&mut self, key: KeyCode) -> &mut Self {
        self.default_up = key;
        self
    }
    pub fn set_down_key(&mut self, key: KeyCode) -> &mut Self {
        self.default_down = key;
        self
    }
    pub fn not_selected_pointer(&mut self, pointer: char) -> &mut Self {
        self.not_selected_pointer = Some(pointer);
        self
    }
    pub fn move_selected_item_forward(&mut self) -> &mut Self {
        self.move_selected_item_forward = true;
        self
    }
    pub fn underline_selected_item(&mut self) -> &mut Self {
        self.underline_selected_item = true;
        self
    }
    pub fn add_up_key(&mut self, key: KeyCode) -> &mut Self {
        self.panic_if_key_is_enter(key);
        self.up_keys.push(key);
        self
    }
    pub fn add_down_key(&mut self, key: KeyCode) -> &mut Self {
        self.panic_if_key_is_enter(key);
        self.down_keys.push(key);
        self
    }
    fn panic_if_key_is_enter(&self, key: KeyCode) {
        if key == KeyCode::Enter {
            panic!("Enter key is not supported as up/down key")
        }
    }
}