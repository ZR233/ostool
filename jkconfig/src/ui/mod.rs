use cursive::{
    Cursive,
    views::{Dialog, SelectView},
};

use crate::data::{AppData, types::ElementType};

pub mod components;

pub fn handle_back(siv: &mut Cursive) {
    if let Some(app) = siv.user_data::<AppData>() {
        if app.current_key.is_empty() {
            handle_quit(siv);
            return;
        }
        app.navigate_back();

        let key = app.key_string();
        siv.pop_layer();

        siv.call_on_all_named(&key, |v: &mut SelectView<ElementType>| {
            flush_menu(&key, v);
        });
    }
}

fn flush_menu(key: &str, s: &mut SelectView<ElementType>) {}

pub fn enter_submenu(siv: &mut Cursive, key: &str) {
    if let Some(app) = siv.user_data::<AppData>() {
        app.enter(key);
    }
}

pub fn handle_quit(siv: &mut Cursive) {
    enter_submenu(siv, "_");
    siv.add_layer(
        Dialog::text("Quit without saving?")
            .title("Quit")
            .button("Back", handle_back)
            .button("Quit", |s| {
                s.quit();
            }),
    );
}
