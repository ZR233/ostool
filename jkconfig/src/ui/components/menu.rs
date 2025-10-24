use cursive::{
    Cursive,
    event::{Event, Key},
    theme::{ColorStyle, Effect, Style},
    utils::markup::StyledString,
    view::{IntoBoxedView, Nameable, Resizable, Scrollable},
    views::{DummyView, LinearLayout, OnEventView, Panel, SelectView, TextView},
};
use log::info;

use crate::{
    data::{AppData, item::ItemType, menu::Menu, types::ElementType},
    ui::{components::icon::ItemDisplay, handle_edit},
};

use super::editors::*;

/// 创建菜单视图
pub fn menu_view(title: &str, path: &str, fields: Vec<ElementType>) -> impl IntoBoxedView {
    let menu_select_name = menu_view_name(path);
    let mut select = SelectView::new();
    select.set_autojump(true);

    select.set_on_select(on_select);
    select.set_on_submit(on_submit);
    menu_select_flush_fields(&mut select, &fields);
    info!("Created menu view for path: {}", path);
    let select = select.with_name(&menu_select_name);

    // 创建标题栏
    let mut title_text = StyledString::new();
    title_text.append_styled("╔═══", ColorStyle::title_primary());
    title_text.append_styled(format!(" {} ", title), Style::from(Effect::Bold));
    title_text.append_styled("═══╗", ColorStyle::title_primary());
    let title_view = TextView::new(title_text).center();

    // 创建路径显示面板
    let path_text = if path.is_empty() {
        let mut styled = StyledString::new();
        styled.append_styled("📂 ", ColorStyle::tertiary());
        styled.append_styled("/ ", Style::from(Effect::Bold));
        styled.append_styled("(Root)", ColorStyle::secondary());
        styled
    } else {
        let mut styled = StyledString::new();
        styled.append_styled("📂 ", ColorStyle::tertiary());
        styled.append_styled(path, Style::from(Effect::Bold));
        styled
    };
    let path_view = TextView::new(path_text).with_name("path_text");

    // 创建详细信息显示区域
    let detail_view = TextView::new(create_status_text())
        .with_name("detail_text")
        .scrollable()
        .fixed_height(8);

    // 创建帮助信息显示区域
    let help_view = TextView::new(create_help_text()).with_name("help_text");

    // 构建主布局
    OnEventView::new(
        LinearLayout::vertical()
            .child(DummyView.fixed_height(1))
            .child(title_view)
            .child(DummyView.fixed_height(1))
            .child(Panel::new(path_view).title("Current Path").full_width())
            .child(DummyView.fixed_height(1))
            .child(
                Panel::new(select.scrollable())
                    .title("Items")
                    .full_width()
                    .min_height(12),
            )
            .child(DummyView.fixed_height(1))
            .child(Panel::new(detail_view).title("Details").full_width())
            .child(DummyView.fixed_height(1))
            .child(
                Panel::new(help_view)
                    .title("Keyboard Shortcuts")
                    .full_width(),
            )
            .child(DummyView.fixed_height(1)),
    )
    .on_event(Event::Char('m'), on_change_set)
    .on_event(Event::Char('M'), on_change_set)
    .on_event(Key::Tab, on_oneof_switch)
    .on_event(Event::Char('c'), on_clear)
    .on_event(Event::Char('C'), on_clear)
}

fn on_clear(s: &mut Cursive) {
    let Some(selected) = menu_selected(s) else {
        return;
    };

    info!("Clear value for selected item {}", selected.key());
    update_selected(s, |elem| elem.set_none());
}

fn update_selected(s: &mut Cursive, f: impl Fn(&mut ElementType)) {
    let Some(selected) = menu_selected(s) else {
        return;
    };

    if let Some(app) = s.user_data::<AppData>()
        && let Some(elem) = app.root.get_mut_by_key(&selected.key())
    {
        f(elem);
        menu_flush(s);
    }
}

fn menu_selected(s: &mut Cursive) -> Option<ElementType> {
    let mut selected = None;
    let name = menu_view_name(&menu_key(s));
    s.call_on_name(&name, |view: &mut SelectView<ElementType>| {
        if let Some(elem) = view.selection() {
            selected = Some(elem.as_ref().clone());
        }
    });

    selected
}

fn on_change_set(s: &mut Cursive) {
    info!("Toggling 'is_set' for menu");

    update_selected(s, |elem| {
        if let ElementType::Menu(menu) = elem {
            if !menu.is_required {
                menu.is_set = !menu.is_set;
            }
            info!("Menu {} is_set toggled to {}", menu.key(), menu.is_set);
        }
    });
}

fn menu_key(s: &mut Cursive) -> String {
    let app = s.user_data::<AppData>().unwrap();
    app.key_string()
}

fn menu_flush(s: &mut Cursive) {
    let key = menu_key(s);
    menu_select_flush(s, &key);
}

pub fn menu_view_name(path: &str) -> String {
    format!("menu_view_{path}")
}

pub fn menu_select_flush(s: &mut Cursive, path: &str) {
    info!("Flushing menu select for path: {}", path);
    let Some(app) = s.user_data::<AppData>() else {
        return;
    };

    let menu = match app.root.get_by_key(path) {
        Some(ElementType::Menu(menu)) => menu,
        Some(ElementType::OneOf(oneof)) => {
            if let Some(selected) = oneof.selected()
                && let ElementType::Menu(menu) = selected
            {
                menu
            } else {
                warn!("No menu selected in OneOf for path: {}", path);
                return;
            }
        }
        _ => {
            warn!("No menu found for path: {}", path);
            return;
        }
    };

    info!("Found menu: {}", menu.key());
    let name = menu_view_name(path);
    let fields = menu.fields();
    s.call_on_name(&name, |view: &mut SelectView<ElementType>| {
        menu_select_flush_fields(view, &fields);
    });
}

fn menu_select_flush_fields(view: &mut SelectView<ElementType>, fields: &[ElementType]) {
    let select_old = view.selected_id();
    view.clear();
    // 为每个字段添加带格式的项
    for field in fields {
        let label = format_item_label(field);
        view.add_item(label, field.clone());
    }
    // 恢复之前的选择位置
    if let Some(idx) = select_old
        && idx < view.len()
    {
        view.set_selection(idx);
    }
    info!("Menu select view flushed with {} fields", fields.len());
}

/// 格式化项目标签，显示类型和当前值
pub fn format_item_label(element: &ElementType) -> StyledString {
    let mut label = StyledString::new();
    label.append_plain(element.icon());
    label.append_plain(" ");
    label.append_styled(&element.title, ColorStyle::title_secondary());
    label.append_plain("  ");
    label.append_styled(element.value(), ColorStyle::secondary());

    label
}

/// 创建帮助文本（在底部状态栏显示）
fn create_help_text() -> StyledString {
    let mut text = StyledString::new();

    // 第一行：基本导航
    text.append_styled("Navigation: ", Style::from(Effect::Underline));
    text.append_plain("\n  ");
    text.append_styled(
        "↑↓/jk",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Move  ");
    text.append_styled(
        "Enter",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Select/Edit  ");
    text.append_styled(
        "Esc",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Back\n");

    // 第二行：编辑操作
    text.append_styled("Actions: ", Style::from(Effect::Underline));
    text.append_plain("\n  ");
    text.append_styled(
        "C",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Clear value  ");
    text.append_styled(
        "M",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Toggle menu  ");
    text.append_styled(
        "Tab",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Switch OneOf\n");

    // 第三行：全局操作
    text.append_styled("Global: ", Style::from(Effect::Underline));
    text.append_plain("\n  ");
    text.append_styled(
        "S",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Save & Exit  ");
    text.append_styled(
        "Q",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Quit  ");
    text.append_styled(
        "~",
        Style::from(Effect::Bold).combine(ColorStyle::tertiary()),
    );
    text.append_plain(" Debug Console");

    text
}

/// 创建状态文本（显示当前项的详细信息）
fn create_status_text() -> String {
    let mut text = String::new();
    text.push_str("╔═══════════════════════════════════════════════\n");
    text.push_str("║\n");
    text.push_str("║  Welcome to JKConfig!\n");
    text.push_str("║\n");
    text.push_str("║  Use ↑↓ or j/k to navigate\n");
    text.push_str("║  Press Enter to select or edit an item\n");
    text.push_str("║\n");
    text.push_str("╚═══════════════════════════════════════════════");
    text
}

/// 当选择项改变时更新详细信息
fn on_select(s: &mut Cursive, item: &ElementType) {
    let detail = match item {
        ElementType::Menu(menu) => {
            let mut text = String::new();
            text.push_str("╔═ Menu ═══════════════════════════════════════\n");
            text.push_str(&format!("║ Title: {}\n", menu.title));
            if let Some(help) = &menu.help {
                text.push_str("║\n");
                for line in help.lines() {
                    text.push_str(&format!("║ {}\n", line));
                }
            }
            text.push_str("║\n");
            text.push_str(&format!("║ Contains {} items\n", menu.children.len()));
            text.push_str("║ Required: ");
            text.push_str(if menu.is_required { "Yes" } else { "No" });
            text.push_str("\n╚═══════════════════════════════════════════════");
            text
        }
        ElementType::OneOf(one_of) => {
            let mut text = String::new();
            text.push_str("╔═ OneOf ══════════════════════════════════════\n");
            text.push_str(&format!("║ Title: {}\n", one_of.title));
            if let Some(help) = &one_of.help {
                text.push_str("║\n");
                for line in help.lines() {
                    text.push_str(&format!("║ {}\n", line));
                }
            }
            text.push_str("║\n");
            text.push_str(&format!("║ Variants: {}\n", one_of.variants.len()));
            if let Some(selected) = one_of.selected() {
                text.push_str(&format!("║ Current: {}\n", selected.title));
            } else {
                text.push_str("║ Current: <Unset>\n");
            }
            text.push_str("║ Tip: Press Tab to switch variants\n");
            text.push_str("╚═══════════════════════════════════════════════");
            text
        }
        ElementType::Item(item) => {
            let mut text = String::new();
            text.push_str("╔═ Item ═══════════════════════════════════════\n");
            text.push_str(&format!("║ Name: {}\n", item.base.title));

            if let Some(help) = &item.base.help {
                text.push_str("║\n");
                for line in help.lines() {
                    text.push_str(&format!("║ {}\n", line));
                }
            }
            text.push_str("║\n");

            match &item.item_type {
                ItemType::Boolean { value, default } => {
                    text.push_str("║ Type: Boolean\n");
                    text.push_str(&format!(
                        "║ Current: {}\n",
                        if *value { "✓ True" } else { "✗ False" }
                    ));
                    text.push_str(&format!(
                        "║ Default: {}\n",
                        if *default { "True" } else { "False" }
                    ));
                    text.push_str("║\n║ Tip: Press Enter to toggle");
                }
                ItemType::String { value, default } => {
                    text.push_str("║ Type: String\n");
                    text.push_str(&format!(
                        "║ Current: {}\n",
                        value
                            .as_ref()
                            .map(|v| format!("\"{}\"", v))
                            .unwrap_or_else(|| "<Empty>".to_string())
                    ));
                    if let Some(d) = default {
                        text.push_str(&format!("║ Default: \"{}\"\n", d));
                    }
                    text.push_str("║\n║ Tip: Press Enter to edit");
                }
                ItemType::Number { value, default } => {
                    text.push_str("║ Type: Number (float)\n");
                    text.push_str(&format!(
                        "║ Current: {}\n",
                        value
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "<Empty>".to_string())
                    ));
                    if let Some(d) = default {
                        text.push_str(&format!("║ Default: {}\n", d));
                    }
                    text.push_str("║\n║ Tip: Press Enter to edit");
                }
                ItemType::Integer { value, default } => {
                    text.push_str("║ Type: Integer\n");
                    text.push_str(&format!(
                        "║ Current: {}\n",
                        value
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "<Empty>".to_string())
                    ));
                    if let Some(d) = default {
                        text.push_str(&format!("║ Default: {}\n", d));
                    }
                    text.push_str("║\n║ Tip: Press Enter to edit");
                }
                ItemType::Enum(enum_item) => {
                    text.push_str("║ Type: Enum\n");
                    text.push_str(&format!("║ Options: {}\n", enum_item.variants.join(", ")));
                    if let Some(val) = enum_item.value_str() {
                        text.push_str(&format!("║ Current: {}\n", val));
                    } else {
                        text.push_str("║ Current: <Unset>\n");
                    }
                    text.push_str("║\n║ Tip: Press Enter to select");
                }
                ItemType::Array(array_item) => {
                    text.push_str("║ Type: Array\n");
                    text.push_str(&format!("║ Element Type: {}\n", array_item.element_type));
                    text.push_str(&format!("║ Count: {}\n", array_item.values.len()));
                    if !array_item.values.is_empty() {
                        text.push_str("║ Values:\n");
                        let max_display = 5;
                        for (idx, val) in array_item.values.iter().take(max_display).enumerate() {
                            text.push_str(&format!("║   [{}] {}\n", idx, val));
                        }
                        if array_item.values.len() > max_display {
                            text.push_str(&format!(
                                "║   ... and {} more\n",
                                array_item.values.len() - max_display
                            ));
                        }
                    } else {
                        text.push_str("║ Values: <Empty>\n");
                    }
                    text.push_str("║\n║ Tip: Enter=Edit, Del=Delete item");
                }
            }
            text.push_str("\n╚═══════════════════════════════════════════════");
            text
        }
    };

    s.call_on_name("detail_text", |v: &mut TextView| {
        v.set_content(detail);
    });
}

pub fn enter_menu(s: &mut Cursive, menu: &Menu) {
    let mut path = String::new();

    if let Some(app) = s.user_data::<AppData>() {
        path = app.key_string();
    }

    let title = menu.title.clone();
    let fields = menu.fields();

    s.add_fullscreen_layer(menu_view(&title, &path, fields));
}

fn enter_elem(s: &mut Cursive, elem: &ElementType) {
    let key = elem.key();
    info!("Entering key: {}, type {}", key, elem.struct_name);
    match elem {
        ElementType::Menu(menu) => {
            info!("Handling Menu: {}", menu.title);
            // 进入子菜单
            enter_menu(s, menu);
        }
        ElementType::OneOf(one_of) => {
            info!("Handling OneOf: {}", one_of.title);
            if let Some(selected) = one_of.selected()
                && let ElementType::Menu(menu) = selected
            {
                // 进入子菜单
                enter_menu(s, menu);
                return;
            }

            // 显示 OneOf 选择对话框
            show_oneof_dialog(s, one_of);
        }
        ElementType::Item(item) => {
            info!("Handling Item: {}", item.base.key());
            // 根据类型显示编辑对话框
            match &item.item_type {
                ItemType::Boolean { .. } => {
                    // Boolean 类型直接切换
                    if let Some(ElementType::Item(b)) =
                        s.user_data::<AppData>().unwrap().root.get_mut_by_key(&key)
                        && let ItemType::Boolean { value, .. } = &mut b.item_type
                    {
                        *value = !*value;
                    }
                    handle_edit(s);
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
                    show_enum_select(s, &item.base.title, enum_item);
                }
                ItemType::Array(array_item) => {
                    show_array_edit(s, &item.base.key(), &item.base.title, &array_item.values);
                }
            }
        }
    }
}

pub fn enter_key(s: &mut Cursive, key: &str) {
    if let Some(app) = s.user_data::<AppData>()
        && let Some(item) = app.root.get_by_key(key).cloned()
    {
        info!("Entering key: {}, got {}", key, item.key());
        app.enter(key);
        enter_elem(s, &item);
    }
}

fn on_oneof_switch(s: &mut Cursive) {
    let Some(selected) = menu_selected(s) else {
        return;
    };

    let ElementType::OneOf(oneof) = selected else {
        return;
    };

    if let Some(app) = s.user_data::<AppData>() {
        let key = oneof.key();
        app.enter(&key);
    }
    show_oneof_dialog(s, &oneof);
}

/// 处理项目选择
fn on_submit(s: &mut Cursive, item: &ElementType) {
    info!("Submitting item: {}", item.key());
    enter_key(s, &item.key());
}
