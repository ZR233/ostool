use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, SelectView, TextView},
};

use crate::data::item::EnumItem;
use crate::ui::components::refresh::refresh_current_menu;

/// 显示枚举选择对话框
pub fn show_enum_select(s: &mut Cursive, key: &str, title: &str, enum_item: &EnumItem) {
    let mut select = SelectView::new();

    for (idx, variant) in enum_item.variants.iter().enumerate() {
        let label = if Some(idx) == enum_item.value {
            format!("(*) {}", variant)
        } else {
            format!("( ) {}", variant)
        };
        select.add_item(label, idx);
    }

    let key = key.to_string();

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!("Select: {}", title)))
                .child(DummyView)
                .child(select.with_name("enum_select").fixed_height(10)),
        )
        .title("Select Option")
        .button("OK", move |s| {
            let selection = s
                .call_on_name("enum_select", |v: &mut SelectView<usize>| v.selection())
                .unwrap();

            if let Some(idx) = selection {
                // TODO: 保存值到 AppData
                s.add_layer(Dialog::info(format!("Set {} = variant {}", key, idx)));
            }
            s.pop_layer();
            // 刷新菜单显示最新值
            refresh_current_menu(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}
