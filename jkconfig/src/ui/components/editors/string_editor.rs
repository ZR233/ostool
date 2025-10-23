use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, EditView, LinearLayout, TextView},
};

use crate::ui::components::refresh::refresh_current_menu;

/// 显示字符串编辑对话框
pub fn show_string_edit(
    s: &mut Cursive,
    key: &str,
    title: &str,
    value: &Option<String>,
    default: &Option<String>,
) {
    let initial = value
        .clone()
        .or_else(|| default.clone())
        .unwrap_or_default();
    let key = key.to_string();

    s.add_layer(
        Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new(format!("Edit: {}", title)))
                .child(DummyView)
                .child(
                    EditView::new()
                        .content(initial)
                        .with_name("edit_value")
                        .fixed_width(50),
                ),
        )
        .title("Edit String")
        .button("OK", move |s| {
            let value = s
                .call_on_name("edit_value", |v: &mut EditView| v.get_content())
                .unwrap();
            // TODO: 保存值到 AppData
            s.add_layer(Dialog::info(format!("Set {} = {}", key, value)));
            s.pop_layer();
            // 刷新菜单显示最新值
            refresh_current_menu(s);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}
