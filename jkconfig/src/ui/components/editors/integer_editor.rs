use cursive::{
    Cursive,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, EditView, LinearLayout, TextView},
};

/// 显示整数编辑对话框
pub fn show_integer_edit(
    s: &mut Cursive,
    key: &str,
    title: &str,
    value: Option<i64>,
    default: Option<i64>,
) {
    let initial = value.or(default).map(|v| v.to_string()).unwrap_or_default();
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
                        .fixed_width(30),
                ),
        )
        .title("Edit Integer")
        .button("OK", move |s| {
            let content = s
                .call_on_name("edit_value", |v: &mut EditView| v.get_content())
                .unwrap();

            match content.parse::<i64>() {
                Ok(_num) => {
                    // TODO: 保存值到 AppData
                    s.add_layer(Dialog::info(format!("Set {} = {}", key, content)));
                    s.pop_layer();
                }
                Err(_) => {
                    s.add_layer(Dialog::info("Invalid integer format!"));
                }
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}
