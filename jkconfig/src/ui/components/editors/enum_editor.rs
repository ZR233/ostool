use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, SelectView, TextView},
};

use crate::{
    data::{
        item::{EnumItem, ItemType},
        types::ElementType,
    },
    ui::handle_back,
};

/// 显示枚举选择对话框
pub fn show_enum_select(s: &mut Cursive, title: &str, enum_item: &EnumItem) {
    let mut select = SelectView::new();

    for (idx, variant) in enum_item.variants.iter().enumerate() {
        let label = if Some(idx) == enum_item.value {
            format!("(*) {}", variant)
        } else {
            format!("( ) {}", variant)
        };
        select.add_item(label, idx);
    }

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!("Select: {}", title)))
                    .child(DummyView)
                    .child(select.with_name("enum_select").fixed_height(10)),
            )
            .title("Select Option")
            .button("OK", on_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, on_ok),
    );
}

fn on_ok(s: &mut Cursive) {
    let selection = s
        .call_on_name("enum_select", |v: &mut SelectView<usize>| v.selection())
        .unwrap();
    let Some(selection) = selection else {
        return;
    };

    if let Some(app) = s.user_data::<crate::data::app_data::AppData>()
        && let Some(ElementType::Item(item)) = app.current_mut()
        && let ItemType::Enum(en) = &mut item.item_type
    {
        en.value = Some(*selection);
    }
    handle_back(s);
}
