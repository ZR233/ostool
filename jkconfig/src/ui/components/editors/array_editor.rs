use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable, Scrollable},
    views::{Dialog, DummyView, EditView, LinearLayout, OnEventView, SelectView, TextView},
};

use crate::{
    data::{item::ItemType, types::ElementType},
    ui::handle_back,
};

/// 显示数组编辑对话框
pub fn show_array_edit(s: &mut Cursive, key: &str, title: &str, values: &[String]) {
    let key_clone = key.to_string();
    let mut select = SelectView::new();

    // Add existing items to the list
    for (idx, value) in values.iter().enumerate() {
        select.add_item(format!("[{}] {}", idx, value), idx);
    }

    // Add "Add new item" option
    select.add_item("+ Add new item", usize::MAX);

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!("Edit Array: {}", title)))
                    .child(DummyView)
                    .child(
                        select
                            .with_name("array_select")
                            .scrollable()
                            .fixed_height(15),
                    ),
            )
            .title("Edit Array")
            .button("OK", move |s| {
                handle_back(s);
            })
            .button("Delete", move |s| {
                on_delete(s);
            }),
        )
        .on_event(Key::Enter, move |s| {
            on_enter(s, &key_clone);
        })
        .on_event(Key::Del, on_delete),
    );
}

fn on_enter(s: &mut Cursive, key: &str) {
    let selection = s
        .call_on_name("array_select", |v: &mut SelectView<usize>| v.selection())
        .unwrap();

    if let Some(idx) = selection {
        if *idx == usize::MAX {
            // Add new item
            show_add_item_dialog(s, key);
        } else {
            // Edit existing item
            show_edit_item_dialog(s, key, *idx);
        }
    }
}

fn on_delete(s: &mut Cursive) {
    let selection = s
        .call_on_name("array_select", |v: &mut SelectView<usize>| v.selection())
        .unwrap();

    if let Some(idx) = selection
        && *idx != usize::MAX
    {
        s.add_layer(
            Dialog::text(format!("Delete item [{}]?", idx))
                .title("Confirm Delete")
                .button("Yes", move |s| {
                    if let Some(app) = s.user_data::<crate::data::app_data::AppData>()
                        && let Some(ElementType::Item(item)) = app.current_mut()
                        && let ItemType::Array(array_item) = &mut item.item_type
                        && *idx < array_item.values.len()
                    {
                        array_item.values.remove(*idx);
                        s.pop_layer(); // Close confirm dialog
                        refresh_array_view(s);
                    }
                })
                .button("No", |s| {
                    s.pop_layer();
                }),
        );
    }
}

fn show_add_item_dialog(s: &mut Cursive, key: &str) {
    let key = key.to_string();
    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Enter new value:"))
                .child(DummyView)
                .child(EditView::new().with_name("new_item_value").fixed_width(50)),
        )
        .title("Add Item")
        .button("OK", move |s| {
            let content = s
                .call_on_name("new_item_value", |v: &mut EditView| v.get_content())
                .unwrap();

            if !content.is_empty() {
                if let Some(app) = s.user_data::<crate::data::app_data::AppData>()
                    && let Some(ElementType::Item(item)) = app.root.get_mut_by_key(&key)
                    && let ItemType::Array(array_item) = &mut item.item_type
                {
                    array_item.values.push(content.to_string());
                    s.pop_layer(); // Close add dialog
                    refresh_array_view(s);
                }
            } else {
                s.add_layer(Dialog::info("Value cannot be empty!").dismiss_button("OK"));
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn show_edit_item_dialog(s: &mut Cursive, key: &str, idx: usize) {
    let key = key.to_string();

    // Get current value
    let current_value = if let Some(app) = s.user_data::<crate::data::app_data::AppData>()
        && let Some(ElementType::Item(item)) = app.root.get_by_key(&key)
        && let ItemType::Array(array_item) = &item.item_type
        && idx < array_item.values.len()
    {
        array_item.values[idx].clone()
    } else {
        String::new()
    };

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!("Edit item [{}]:", idx)))
                .child(DummyView)
                .child(
                    EditView::new()
                        .content(current_value)
                        .with_name("edit_item_value")
                        .fixed_width(50),
                ),
        )
        .title("Edit Item")
        .button("OK", move |s| {
            let content = s
                .call_on_name("edit_item_value", |v: &mut EditView| v.get_content())
                .unwrap();

            if !content.is_empty() {
                if let Some(app) = s.user_data::<crate::data::app_data::AppData>()
                    && let Some(ElementType::Item(item)) = app.root.get_mut_by_key(&key)
                    && let ItemType::Array(array_item) = &mut item.item_type
                    && idx < array_item.values.len()
                {
                    array_item.values[idx] = content.to_string();
                    s.pop_layer(); // Close edit dialog
                    refresh_array_view(s);
                }
            } else {
                s.add_layer(Dialog::info("Value cannot be empty!").dismiss_button("OK"));
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn refresh_array_view(s: &mut Cursive) {
    // Get current array values
    let values = if let Some(app) = s.user_data::<crate::data::app_data::AppData>()
        && let Some(ElementType::Item(item)) = app.current()
        && let ItemType::Array(array_item) = &item.item_type
    {
        array_item.values.clone()
    } else {
        return;
    };

    // Update the select view
    s.call_on_name("array_select", |view: &mut SelectView<usize>| {
        view.clear();
        for (idx, value) in values.iter().enumerate() {
            view.add_item(format!("[{}] {}", idx, value), idx);
        }
        view.add_item("+ Add new item", usize::MAX);
    });
}
