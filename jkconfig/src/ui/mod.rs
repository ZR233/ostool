use cursive::{Cursive, views::Dialog};

use crate::{data::AppData, ui::components::menu::menu_select_flush};

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

        menu_select_flush(siv, &key);
    }
}

pub fn handle_edit(siv: &mut Cursive) {
    if let Some(app) = siv.user_data::<AppData>() {
        app.navigate_back();
        let key = app.key_string();
        menu_select_flush(siv, &key);
    }
}

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
