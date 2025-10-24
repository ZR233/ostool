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
    ui::handle_edit,
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

    // 创建路径显示面板
    let path_text = if path.is_empty() {
        StyledString::styled("Path: / (Root)", ColorStyle::tertiary())
    } else {
        let mut styled = StyledString::new();
        styled.append_styled("Path: ", ColorStyle::secondary());
        styled.append_styled(path, ColorStyle::tertiary());
        styled
    };
    let path_view = TextView::new(path_text).with_name("path_text");

    // 创建帮助信息显示区域
    let help_view = TextView::new(create_help_text()).with_name("help_text");

    // 创建详细信息显示区域
    let detail_view = TextView::new(create_status_text())
        .with_name("detail_text")
        .scrollable()
        .fixed_height(5);

    // 构建主布局
    OnEventView::new(
        LinearLayout::vertical()
            .child(TextView::new(title).center())
            .child(Panel::new(path_view).full_width())
            .child(DummyView)
            .child(select.scrollable().full_width().min_height(10))
            .child(DummyView)
            .child(Panel::new(detail_view).title("Help").full_width())
            .child(DummyView)
            .child(Panel::new(help_view).full_width()),
    )
    .on_event(Event::Char('m'), on_change_set)
    .on_event(Key::Tab, move |s| on_oneof_switch(s, &menu_select_name))
}

fn on_change_set(s: &mut Cursive) {
    if let Some(app) = s.user_data::<AppData>()
        && let Some(ElementType::Menu(v)) = &app.select_field
    {
        let key = v.key();
        let ElementType::Menu(v) = app.root.get_mut_by_key(&key).unwrap() else {
            return;
        };
        if !v.is_required {
            v.is_set = !v.is_set;
        }
        menu_select_flush(s, &key);
    }
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
                info!("No menu selected in OneOf for path: {}", path);
                return;
            }
        }
        _ => {
            info!("No menu found for path: {}", path);
            return;
        }
    };

    info!("Found menu: {}", menu.key());
    let name = menu_view_name(path);
    let fields = menu.children.values().cloned().collect::<Vec<_>>();
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
}

// Emoji 状态常量定义
const EMOJI_REQUIRED: &str = "⭕";
const EMOJI_OPTIONAL: &str = "  "; // 等长空格保持对齐
const EMOJI_SET: &str = "✅";
const EMOJI_UNSET: &str = "  ";
const EMOJI_MENU: &str = "📁";
const EMOJI_ONEOF: &str = "🎛️";
const EMOJI_STRING: &str = "📝";
const EMOJI_NUMBER: &str = "🔢";
const EMOJI_INTEGER: &str = "☑️";
const EMOJI_ENUM: &str = "📋";
const EMOJI_BOOL_TRUE: &str = "✅";
const EMOJI_BOOL_FALSE: &str = "❌";

/// 获取 required 状态的 emoji
fn get_required_emoji(is_required: bool) -> &'static str {
    if is_required {
        EMOJI_REQUIRED
    } else {
        EMOJI_OPTIONAL
    }
}

/// 获取赋值状态的 emoji
fn get_value_emoji(element: &ElementType) -> &'static str {
    if element.is_none() {
        EMOJI_UNSET
    } else {
        EMOJI_SET
    }
}

/// 获取类型图标的 emoji（Boolean 类型特殊处理，直接返回状态）
fn get_type_emoji(element: &ElementType) -> Option<&'static str> {
    match element {
        ElementType::Menu(_) => Some(EMOJI_MENU),
        ElementType::OneOf(_) => Some(EMOJI_ONEOF),
        ElementType::Item(item) => {
            match &item.item_type {
                ItemType::Boolean { .. } => {
                    // Boolean 类型直接用状态 emoji，不需要额外的类型图标
                    None
                }
                ItemType::String { .. } => Some(EMOJI_STRING),
                ItemType::Number { .. } => Some(EMOJI_NUMBER),
                ItemType::Integer { .. } => Some(EMOJI_INTEGER),
                ItemType::Enum(_) => Some(EMOJI_ENUM),
            }
        }
    }
}

/// 获取 Boolean 类型的状态 emoji
fn get_boolean_emoji(value: bool) -> &'static str {
    if value {
        EMOJI_BOOL_TRUE
    } else {
        EMOJI_BOOL_FALSE
    }
}

/// 格式化项目标签，显示类型和当前值
pub fn format_item_label(element: &ElementType) -> StyledString {
    let mut label = StyledString::new();

    match element {
        ElementType::Menu(menu) => {
            // 菜单项：[required][value][type] 标题
            label.append_styled(get_required_emoji(menu.is_required), ColorStyle::highlight());
            label.append_styled(get_value_emoji(element), ColorStyle::title_secondary());
            label.append_styled(EMOJI_MENU, ColorStyle::tertiary());
            label.append_plain(" ");
            label.append_plain(&menu.title);

            if menu.is_required {
                label.append_styled(" *", ColorStyle::highlight());
            }
        }
        ElementType::OneOf(one_of) => {
            // OneOf 选择项：[required][value][type] 标题 = 当前选择
            label.append_styled(get_required_emoji(one_of.is_required), ColorStyle::highlight());
            label.append_styled(get_value_emoji(element), ColorStyle::title_secondary());
            label.append_styled(EMOJI_ONEOF, ColorStyle::tertiary());
            label.append_plain(" ");
            label.append_plain(&one_of.title);

            if let Some(selected) = one_of.selected() {
                label.append_styled(" = ", Style::from(ColorStyle::secondary()));
                label.append_styled(&selected.struct_name, ColorStyle::title_secondary());
            }

            if one_of.is_required {
                label.append_styled(" *", ColorStyle::highlight());
            }
        }
        ElementType::Item(item) => {
            // Boolean 类型特殊处理：直接用 ✅/❌ 表示状态
            if let ItemType::Boolean { value, .. } = &item.item_type {
                label.append_styled(get_required_emoji(item.base.is_required), ColorStyle::highlight());
                label.append_styled("  ", ColorStyle::title_secondary()); // Boolean 不需要赋值状态标识
                label.append_styled(get_boolean_emoji(*value), ColorStyle::primary());
                label.append_plain(" ");
                label.append_plain(&item.base.title);

                if item.base.is_required {
                    label.append_styled(" *", ColorStyle::highlight());
                }
                return label;
            }

            // 其他类型：[required][value][type] 标题 = 当前值
            label.append_styled(get_required_emoji(item.base.is_required), ColorStyle::highlight());
            label.append_styled(get_value_emoji(element), ColorStyle::title_secondary());

            if let Some(type_emoji) = get_type_emoji(element) {
                label.append_styled(type_emoji, ColorStyle::tertiary());
            }
            label.append_plain(" ");
            label.append_plain(&item.base.title);

            // 添加当前值显示
            let value_str = match &item.item_type {
                ItemType::String { value, .. } => {
                    value.as_ref().map(|v| {
                        if v.len() > 30 {
                            format!("\"{}...\"", &v[..27])
                        } else {
                            format!("\"{}\"", v)
                        }
                    })
                }
                ItemType::Number { value, .. } => {
                    value.map(|v| format!("{:.2}", v))
                }
                ItemType::Integer { value, .. } => {
                    value.map(|v| v.to_string())
                }
                ItemType::Enum(enum_item) => {
                    enum_item.value_str().map(|s| s.to_string())
                }
                _ => None,
            };

            if let Some(val) = value_str {
                label.append_styled(" = ", Style::from(ColorStyle::secondary()));
                label.append_styled(val, ColorStyle::title_secondary());
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
    info!("Selected item: {}", item.key());
    if let Some(app) = s.user_data::<AppData>() {
        app.select_field = Some(item.clone());
    }

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

pub fn enter_menu(s: &mut Cursive, menu: &Menu) {
    let mut path = String::new();

    if let Some(app) = s.user_data::<AppData>() {
        path = app.key_string();
    }

    let title = menu.title.clone();
    let fields: Vec<ElementType> = menu.children.values().cloned().collect();

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

fn on_oneof_switch(s: &mut Cursive, name: &str) {
    let mut oneof = None;

    s.call_on_name(name, |view: &mut SelectView<ElementType>| {
        let selected = view.selection();
        if let Some(elem) = selected
            && let ElementType::OneOf(_one_of) = elem.as_ref()
        {
            oneof = Some(elem.as_ref().clone());
        }
    });

    if let Some(ElementType::OneOf(one_of)) = &oneof {
        if let Some(app) = s.user_data::<AppData>() {
            let key = one_of.key();
            app.enter(&key);
        }
        show_oneof_dialog(s, one_of);
    }
}

/// 处理项目选择
fn on_submit(s: &mut Cursive, item: &ElementType) {
    info!("Submitting item: {}", item.key());
    enter_key(s, &item.key());
}
