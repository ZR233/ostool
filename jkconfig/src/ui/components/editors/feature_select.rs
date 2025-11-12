use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextView},
};
use std::{collections::HashMap, path::Path};
use std::{collections::HashSet, sync::Arc};

use crate::{
    data::{app_data::AppData, item::ItemType, types::ElementType},
    ui::{
        components::editors::multi_select_editor::{MultiSelectItem, show_multi_select},
        handle_back,
    },
};

pub fn show_feature_select(s: &mut Cursive, package: &str, manifest_path: &Path) {
    if let Ok(metadata) = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
    {
        match metadata.packages.iter().find(|p| p.name == package) {
            Some(pkg) => {
                let features: Vec<String> = pkg.features.keys().cloned().collect();

                let old = if let Some(app) = s.user_data::<AppData>() {
                    if let Some(ElementType::Item(item)) = app.current() {
                        if let ItemType::Array(array) = &item.item_type {
                            array.values.clone()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                let mut selected: HashSet<String> = old.into_iter().collect();
                let mut select = ;

                // 添加所有选项到SelectView
                           

                s.add_fullscreen_layer(OnEventView::new(Dialog::around(
                    LinearLayout::vertical()
                        .child(TextView::new("Select Feature".to_string()))
                        .child(
                            TextView::new("(Press Enter to select feature)")
                                .style(cursive::theme::ColorStyle::secondary()),
                        )
                        .child(DummyView)
                        .child(
                            ScrollView::new(select.with_name("feature_select")).fixed_height(20),
                        ),
                )));

                // show_multi_select(s, &format!("Features for {}", package), &multi_select_item);

                // // 在multi-select界面关闭后，更新selected集合
                // if let Some(app) = s.user_data::<AppData>() {
                //     if let Some((key, temp_value)) = &app.temp_data {
                //         if key == "selected_features" {
                //             if let Ok(new_selected) =
                //                 serde_json::from_value::<HashSet<String>>(temp_value.clone())
                //             {
                //                 selected = new_selected;
                //             }
                //         }
                //     }
                // }
            }
            None => {
                let mut dialog =
                    Dialog::info(format!("Package '{}' not found in Cargo.toml", package));
                dialog
                    .buttons_mut()
                    .next()
                    .unwrap()
                    .set_callback(handle_back);

                s.add_layer(dialog);
            }
        }
    }
}

/// 显示依赖项选择对话框
pub fn show_depend_select(
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
        .on_event(Key::Enter, on_depend_select)
        .on_event(Key::Right, |s| {
            s.on_event(cursive::event::Event::Key(cursive::event::Key::Tab));
        }),
    );
}

/// 处理依赖项选择
fn on_depend_select(s: &mut Cursive) {
    // 获取当前选中的依赖项
    let selection = s
        .call_on_name("depend_select", |v: &mut SelectView<Arc<String>>| {
            v.selection()
        })
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

        // 获取选中依赖项的features
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

/// 确认依赖项选择
fn on_depend_ok(s: &mut Cursive) {
    // 清理临时数据
    if let Some(app) = s.user_data::<AppData>() {
        app.temp_data = None;
    }
    handle_back(s);
}
