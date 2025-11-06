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

/// 显示依赖项features编辑器
pub fn show_depend_features_editor(
    s: &mut Cursive,
    title: &str,
    depend_names: &[String], // 依赖项名称列表
    depend_features_map: &HashMap<String, Vec<String>>
) {
    let mut select = SelectView::new();

    // 添加所有依赖项到SelectView
    for (i, depend_name) in depend_names.iter().enumerate() {
        select.add_item(depend_name.clone(), Arc::new(i));
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
                    .child(TextView::new(format!("Edit Dependency Features: {}", title)))
                    .child(
                        TextView::new("(Press Enter to edit dependency features)")
                            .style(cursive::theme::ColorStyle::secondary()),
                    )
                    .child(DummyView)
                    .child(ScrollView::new(select.with_name("depend_features_select")).fixed_height(20)),
            )
            .title("Dependencies")
            .button("OK", on_depend_features_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, on_depend_feature_select),
    );
}

/// 处理依赖项features选择
fn on_depend_feature_select(s: &mut Cursive) {
    // 获取当前选中的依赖项
    let selection = s
        .call_on_name("depend_features_select", |v: &mut SelectView<Arc<usize>>| v.selection())
        .unwrap();

    if let Some(selection_index) = selection {
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

        // 获取选中依赖项的features（这里需要根据实际数据结构进行调整）
        let depend_names: Vec<String> = depend_features_map.keys().cloned().collect();
        if let Some(depend_name) = depend_names.get(**selection_index) {
            if let Some(features) = depend_features_map.get(depend_name) {
                // 创建MultiSelectItem用于显示features选择
                let multi_select_item = MultiSelectItem {
                    variants: features.clone(),
                    selected_indices: Vec::new(), // 默认不选择任何项
                };
                
                // 保存当前选中的依赖项索引，以便在features选择后更新
                if let Some(app) = s.user_data::<AppData>() {
                    app.temp_data = Some((
                        "current_depend_index".to_string(),
                        serde_json::to_value(**selection_index).unwrap(),
                    ));
                }
                
                // 显示features多选界面
                show_multi_select(s, &format!("Features for {}", depend_name), &multi_select_item);
            }
        }
    }
}

/// 确认依赖项features编辑
fn on_depend_features_ok(s: &mut Cursive) {
    handle_back(s);
}