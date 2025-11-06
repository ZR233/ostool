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
    ui::{components::editors::multi_select_editor::{show_multi_select, MultiSelectItem}, handle_back},
};

/// 显示依赖项选择对话框
pub fn show_depend_select(
    s: &mut Cursive, 
    title: &str, 
    depend_names: &[String], 
    depend_features_map: &HashMap<String, Vec<String>>
) {
    let mut select = SelectView::new();

    // 添加所有依赖项到SelectView
    for depend_name in depend_names {
        select.add_item(depend_name.clone(), Arc::new(depend_name.clone()));
    }

    // 保存依赖项features映射到应用数据中
    if let Some(app) = s.user_data::<AppData>() {
        // 保存依赖项features映射
        app.temp_data = Some((
            "depend_features_map".to_string(),
            serde_json::to_value(depend_features_map.clone()).unwrap(),
        ));
    }

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!("Select Dependency: {}", title)))
                    .child(
                        TextView::new("(Press Enter to select dependency features)")
                            .style(cursive::theme::ColorStyle::secondary()),
                    )
                    .child(DummyView)
                    .child(ScrollView::new(select.with_name("depend_select")).fixed_height(20)),
            )
            .title("Dependency Selection")
            .button("OK", on_depend_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, on_depend_select),
    );
}

/// 处理依赖项选择
fn on_depend_select(s: &mut Cursive) {
    // 获取当前选中的依赖项
    let selection = s
        .call_on_name("depend_select", |v: &mut SelectView<Arc<String>>| v.selection())
        .unwrap();

    if let Some(selection_name) = selection {
        // 获取保存的依赖项features映射
        let mut depend_features_map = HashMap::new();
        
        if let Some(app) = s.user_data::<AppData>() {
            if let Some((_, temp_value)) = &app.temp_data {
                // 尝试从temp_data中获取保存的依赖项features映射
                if let Ok(map) = serde_json::from_value::<HashMap<String, Vec<String>>>(temp_value.clone()) {
                    depend_features_map = map;
                }
            }
        }

        // 获取选中依赖项的features
        if let Some(features) = depend_features_map.get(&**selection_name) {
            // 创建MultiSelectItem用于显示features选择
            let multi_select_item = MultiSelectItem {
                variants: features.clone(),
                selected_indices: Vec::new(), // 默认不选择任何项
            };
            
            // 保存当前选中的依赖项名称，以便在features选择后更新
            if let Some(app) = s.user_data::<AppData>() {
                app.temp_data = Some((
                    "current_depend".to_string(),
                    serde_json::to_value((**selection_name).clone()).unwrap(),
                ));
            }
            
            // 显示features多选界面
            show_multi_select(s, &format!("Features for {}", selection_name), &multi_select_item);
        }
    }
}

/// 确认依赖项选择
fn on_depend_ok(s: &mut Cursive) {
    handle_back(s);
}