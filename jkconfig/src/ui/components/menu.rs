use cursive::{
    Cursive,
    theme::{ColorStyle, Effect, Style},
    utils::markup::StyledString,
    view::{IntoBoxedView, Nameable, Resizable, Scrollable},
    views::{Dialog, DummyView, EditView, LinearLayout, Panel, SelectView, TextView},
};

use crate::data::{
    item::{EnumItem, ItemType},
    types::ElementType,
};

/// 创建菜单视图
pub fn menu_view(title: &str, fields: Vec<ElementType>) -> impl IntoBoxedView {
    let mut select = SelectView::new();
    select.set_autojump(true);

    // 为每个字段添加带格式的项
    for field in fields {
        let label = format_item_label(&field);
        select.add_item(label, field);
    }

    select.set_on_select(on_select);

    // 创建帮助信息显示区域
    let help_view = TextView::new(create_help_text()).with_name("help_text");

    // 创建详细信息显示区域
    let detail_view = TextView::new(create_status_text())
        .with_name("detail_text")
        .scrollable()
        .fixed_height(5);

    // 构建主布局
    let content = LinearLayout::vertical()
        .child(
            select
                .on_submit(on_submit)
                .with_name("main_select")
                .scrollable()
                .full_width()
                .min_height(10),
        )
        .child(DummyView)
        .child(Panel::new(detail_view).title("Help").full_width())
        .child(DummyView)
        .child(Panel::new(help_view).full_width());

    Dialog::around(content)
        .title(title)
        .button("Back (Esc)", |s| {
            s.pop_layer();
        })
        .full_screen()
}

/// 格式化项目标签，显示类型和当前值
fn format_item_label(element: &ElementType) -> StyledString {
    let mut label = StyledString::new();

    match element {
        ElementType::Menu(menu) => {
            // 菜单项：显示 [>] 符号
            label.append_styled("[>] ", ColorStyle::secondary());
            label.append_plain(&menu.title);

            if menu.is_required {
                label.append_styled(" *", ColorStyle::highlight());
            }
        }
        ElementType::OneOf(one_of) => {
            // OneOf 选择项
            label.append_styled("[?] ", ColorStyle::tertiary());
            label.append_plain(&one_of.title);

            if let Some(selected) = one_of.selected() {
                label.append_styled(" = ", Style::from(ColorStyle::secondary()));
                label.append_styled(&selected.title, ColorStyle::title_secondary());
            }

            if one_of.is_required {
                label.append_styled(" *", ColorStyle::highlight());
            }
        }
        ElementType::Item(item) => {
            // 根据项目类型显示不同的前缀和值
            let (prefix, value_str) = match &item.item_type {
                ItemType::Boolean { value, .. } => {
                    let checkbox = if *value { "[X]" } else { "[ ]" };
                    (checkbox, String::new())
                }
                ItemType::String { value, .. } => {
                    let val = value
                        .as_ref()
                        .map(|v| {
                            if v.len() > 30 {
                                format!("\"{}...\"", &v[..27])
                            } else {
                                format!("\"{}\"", v)
                            }
                        })
                        .unwrap_or_else(|| "<empty>".to_string());
                    (" S ", val)
                }
                ItemType::Number { value, .. } => {
                    let val = value
                        .map(|v| format!("{:.2}", v))
                        .unwrap_or_else(|| "<unset>".to_string());
                    (" # ", val)
                }
                ItemType::Integer { value, .. } => {
                    let val = value
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "<unset>".to_string());
                    (" N ", val)
                }
                ItemType::Enum(enum_item) => {
                    let val = enum_item.value_str().unwrap_or("<select>").to_string();
                    (" E ", val)
                }
            };

            label.append_styled(prefix, ColorStyle::secondary());
            label.append_plain(&item.base.title);

            if !value_str.is_empty() {
                label.append_styled(" = ", Style::from(ColorStyle::secondary()));
                label.append_styled(value_str, ColorStyle::title_secondary());
            }

            if item.base.is_required {
                label.append_styled(" *", ColorStyle::highlight());
            }
        }
    }

    label
}

/// 创建帮助文本（在底部状态栏显示）
fn create_help_text() -> StyledString {
    let mut text = StyledString::new();
    text.append_styled("Enter", Style::from(Effect::Bold));
    text.append_plain(": Select/Edit  ");
    text.append_styled("↑↓", Style::from(Effect::Bold));
    text.append_plain(": Navigate  ");
    text.append_styled("Esc", Style::from(Effect::Bold));
    text.append_plain(": Back  ");
    text.append_styled("S", Style::from(Effect::Bold));
    text.append_plain(": Save  ");
    text.append_styled("Q", Style::from(Effect::Bold));
    text.append_plain(": Quit");
    text
}

/// 创建状态文本（显示当前项的详细信息）
fn create_status_text() -> &'static str {
    "Select an item to view details"
}

/// 当选择项改变时更新详细信息
fn on_select(s: &mut Cursive, item: &ElementType) {
    let detail = match item {
        ElementType::Menu(menu) => {
            let mut text = format!("Menu: {}\n", menu.title);
            if let Some(help) = &menu.help {
                text.push_str(&format!("\n{}", help));
            }
            text.push_str(&format!("\n\nContains {} items", menu.children.len()));
            text
        }
        ElementType::OneOf(one_of) => {
            let mut text = format!("OneOf: {}\n", one_of.title);
            if let Some(help) = &one_of.help {
                text.push_str(&format!("\n{}", help));
            }
            text.push_str(&format!("\n\nVariants: {}", one_of.variants.len()));
            if let Some(selected) = one_of.selected() {
                text.push_str(&format!("\nCurrent: {}", selected.title));
            }
            text
        }
        ElementType::Item(item) => {
            let mut text = format!("Item: {}\n", item.base.title);
            if let Some(help) = &item.base.help {
                text.push_str(&format!("\n{}", help));
            }

            match &item.item_type {
                ItemType::Boolean { value, default } => {
                    text.push_str("\n\nType: Boolean");
                    text.push_str(&format!("\nCurrent: {}", value));
                    text.push_str(&format!("\nDefault: {}", default));
                }
                ItemType::String { value, default } => {
                    text.push_str("\n\nType: String");
                    if let Some(v) = value {
                        text.push_str(&format!("\nCurrent: \"{}\"", v));
                    }
                    if let Some(d) = default {
                        text.push_str(&format!("\nDefault: \"{}\"", d));
                    }
                }
                ItemType::Number { value, default } => {
                    text.push_str("\n\nType: Number");
                    if let Some(v) = value {
                        text.push_str(&format!("\nCurrent: {}", v));
                    }
                    if let Some(d) = default {
                        text.push_str(&format!("\nDefault: {}", d));
                    }
                }
                ItemType::Integer { value, default } => {
                    text.push_str("\n\nType: Integer");
                    if let Some(v) = value {
                        text.push_str(&format!("\nCurrent: {}", v));
                    }
                    if let Some(d) = default {
                        text.push_str(&format!("\nDefault: {}", d));
                    }
                }
                ItemType::Enum(enum_item) => {
                    text.push_str("\n\nType: Enum");
                    text.push_str(&format!("\nOptions: {}", enum_item.variants.join(", ")));
                    if let Some(val) = enum_item.value_str() {
                        text.push_str(&format!("\nCurrent: {}", val));
                    }
                }
            }
            text
        }
    };

    s.call_on_name("detail_text", |v: &mut TextView| {
        v.set_content(detail);
    });
}

/// 处理项目选择
fn on_submit(s: &mut Cursive, item: &ElementType) {
    match item {
        ElementType::Menu(menu) => {
            // 进入子菜单
            let title = menu.title.clone();
            let fields = menu.children.values().cloned().collect();
            s.add_fullscreen_layer(menu_view(&title, fields));
        }
        ElementType::OneOf(one_of) => {
            // 显示 OneOf 选择对话框
            show_oneof_dialog(s, one_of);
        }
        ElementType::Item(item) => {
            // 根据类型显示编辑对话框
            match &item.item_type {
                ItemType::Boolean { .. } => {
                    // Boolean 类型直接切换
                    toggle_boolean(s, &item.base.key());
                }
                ItemType::String { value, default } => {
                    show_string_edit(s, &item.base.key(), &item.base.title, value, default);
                }
                ItemType::Number { value, default } => {
                    show_number_edit(s, &item.base.key(), &item.base.title, *value, *default);
                }
                ItemType::Integer { value, default } => {
                    show_integer_edit(s, &item.base.key(), &item.base.title, *value, *default);
                }
                ItemType::Enum(enum_item) => {
                    show_enum_select(s, &item.base.key(), &item.base.title, enum_item);
                }
            }
        }
    }
}

/// 切换布尔值
fn toggle_boolean(s: &mut Cursive, key: &str) {
    // TODO: 实现布尔值切换逻辑
    // 需要访问 AppData 并更新值
    s.add_layer(Dialog::info(format!("Toggle boolean: {}", key)));
}

/// 显示字符串编辑对话框
fn show_string_edit(
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
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

/// 显示数字编辑对话框
fn show_number_edit(
    s: &mut Cursive,
    key: &str,
    title: &str,
    value: Option<f64>,
    default: Option<f64>,
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
        .title("Edit Number")
        .button("OK", move |s| {
            let content = s
                .call_on_name("edit_value", |v: &mut EditView| v.get_content())
                .unwrap();

            match content.parse::<f64>() {
                Ok(_num) => {
                    // TODO: 保存值到 AppData
                    s.add_layer(Dialog::info(format!("Set {} = {}", key, content)));
                    s.pop_layer();
                }
                Err(_) => {
                    s.add_layer(Dialog::info("Invalid number format!"));
                }
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

/// 显示整数编辑对话框
fn show_integer_edit(
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

/// 显示枚举选择对话框
fn show_enum_select(s: &mut Cursive, key: &str, title: &str, enum_item: &EnumItem) {
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
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

/// 显示 OneOf 选择对话框
fn show_oneof_dialog(s: &mut Cursive, one_of: &crate::data::oneof::OneOf) {
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
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}
