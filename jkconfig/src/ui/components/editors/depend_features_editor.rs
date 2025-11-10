use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextView},
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    data::app_data::AppData,
    ui::{
        components::editors::multi_select_editor::{MultiSelectItem, show_multi_select},
        handle_back,
    },
};

/// 显示依赖项features编辑器
pub fn show_depend_features_editor(
    s: &mut Cursive,
    title: &str,
    depend_names: &[String],
    depend_features_map: &HashMap<String, Vec<String>>,
) {
    let mut select = SelectView::new();

    // 创建排序后的依赖项列表
    let mut sorted_depend_names: Vec<String> = depend_names.to_vec();
    sorted_depend_names.sort();

    // 添加所有依赖项到SelectView（按名称排序）
    for depend_name in &sorted_depend_names {
        select.add_item(depend_name.clone(), Arc::new(depend_name.clone()));
    }

    // 保存依赖项features映射到应用数据中
    if let Some(app) = s.user_data::<AppData>() {
        app.temp_data = Some((
            "depend_features_map".to_string(),
            serde_json::to_value(depend_features_map.clone()).unwrap(),
        ));
    }

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!(
                        "Edit Dependency Features: {}",
                        title
                    )))
                    .child(
                        TextView::new("(Press Enter to edit dependency features)")
                            .style(cursive::theme::ColorStyle::secondary()),
                    )
                    .child(DummyView)
                    .child(
                        ScrollView::new(select.with_name("depend_features_select"))
                            .fixed_height(20),
                    ),
            )
            .title("Dependencies")
            .button("OK", on_depend_features_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, on_depend_feature_select)
        .on_event(Key::Right, |s| {
            s.on_event(cursive::event::Event::Key(cursive::event::Key::Tab));
        }),
    );
}

/// 处理依赖项features选择
fn on_depend_feature_select(s: &mut Cursive) {
    // 获取当前选中的依赖项
    let selection = s
        .call_on_name(
            "depend_features_select",
            |v: &mut SelectView<Arc<String>>| v.selection(),
        )
        .unwrap();

    if let Some(selection_name) = selection {
        // 获取保存的依赖项features映射
        let mut depend_features_map = HashMap::new();

        if let Some(app) = s.user_data::<AppData>() {
            if let Some((key, temp_value)) = &app.temp_data {
                // 检查是否是依赖项features映射数据
                if key == "depend_features_map" {
                    // 尝试从temp_data中获取保存的依赖项features映射
                    if let Ok(map) =
                        serde_json::from_value::<HashMap<String, Vec<String>>>(temp_value.clone())
                    {
                        depend_features_map = map;
                    }
                }
            }
        }

        // 如果没有从temp_data获取到映射，尝试从depend_features_callback获取
        if depend_features_map.is_empty() {
            if let Some(app) = s.user_data::<AppData>() {
                if let Some(callback) = &app.depend_features_callback {
                    let get_depend_features = || callback();
                    if let Ok(features_map) =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(get_depend_features))
                    {
                        depend_features_map = features_map;
                    }
                }
            }
        }

        // 获取选中依赖项的features（使用依赖项名称而不是索引）
        if let Some(features) = depend_features_map.get(&**selection_name) {
            // 创建MultiSelectItem用于显示features选择
            let multi_select_item = MultiSelectItem {
                variants: features.clone(),
                selected_indices: Vec::new(),
            };

            // 保存当前选中的依赖项名称，以便在features选择后更新
            if let Some(app) = s.user_data::<AppData>() {
                app.temp_data = Some((
                    "current_depend".to_string(),
                    serde_json::to_value((**selection_name).clone()).unwrap(),
                ));
            }

            // 显示features多选界面
            show_multi_select(
                s,
                &format!("Features for {}", selection_name),
                &multi_select_item,
            );
        }
    }
}

/// 确认依赖项features编辑
fn on_depend_features_ok(s: &mut Cursive) {
    // 清理临时数据
    if let Some(app) = s.user_data::<AppData>() {
        app.temp_data = None;
    }
    handle_back(s);
}
