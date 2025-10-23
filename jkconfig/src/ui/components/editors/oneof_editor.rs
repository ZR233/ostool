use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, SelectView, TextView},
};

use crate::data::oneof::OneOf;
use crate::ui::components::refresh::refresh_current_menu;

/// 显示 OneOf 选择对话框
pub fn show_oneof_dialog(s: &mut Cursive, one_of: &OneOf) {
    let mut select = SelectView::new();

    for (idx, variant) in one_of.variants.iter().enumerate() {
        let label = if Some(idx) == one_of.selected_index {
            format!("(*) {}", variant.title)
        } else {
            format!("( ) {}", variant.title)
        };
        select.add_item(label, idx);
    }

    let key = one_of.base.key();

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!("Select variant: {}", one_of.title)))
                .child(DummyView)
                .child(select.with_name("oneof_select").fixed_height(10)),
        )
        .title("Select One Of")
        .button("OK", move |s| {
            let selection = s
                .call_on_name("oneof_select", |v: &mut SelectView<usize>| v.selection())
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
