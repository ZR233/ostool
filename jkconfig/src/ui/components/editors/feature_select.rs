use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextView},
};
use std::{collections::HashMap, path::Path, sync::Arc};

use crate::{
    data::{app_data::AppData, item::ItemType, types::ElementType},
    ui::{
        components::editors::multi_select_editor::{
            DepItem, ExtendedMultiSelectItem, MultiSelectItem, show_extended_multi_select,
            show_multi_select,
        },
        handle_back,
    },
};

/// 显示特性选择对话框，支持主要特性和依赖项特性的选择
///
/// # 参数
/// - `s`: Cursive UI 实例
/// - `package`: 要选择特性的包名
/// - `manifest_path`: Cargo.toml 文件路径
/// - `deps_filter`: 可选的依赖项过滤器
///   - `Some(&[String])`: 只显示指定的依赖项
///   - `None`: 显示所有有 features 的依赖项
///
/// # 功能
/// - 显示当前包的所有 features
/// - 显示依赖项的 features（可过滤）
/// - 支持多选操作
/// - 依赖项特性以 "dep_name/feature" 格式保存
pub fn show_feature_select(
    s: &mut Cursive,
    package: &str,
    manifest_path: &Path,
    deps_filter: Option<&[String]>,
) {
    if let Ok(metadata) = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
    {
        match metadata.packages.iter().find(|p| p.name == package) {
            Some(pkg) => {
                let all_features: Vec<String> = pkg.features.keys().cloned().collect();

                // 获取当前选中的特性列表
                let selected_values = if let Some(app) = s.user_data::<AppData>() {
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

                // 分离主要特性和依赖项特性
                let mut main_features = Vec::new();
                let mut dep_selected_features = HashMap::new();

                for value in &selected_values {
                    if value.contains('/') {
                        // 这是依赖项特性，格式为 "dep_name/feature"
                        if let Some((dep_name, feature)) = value.split_once('/') {
                            dep_selected_features
                                .entry(dep_name.to_string())
                                .or_insert_with(Vec::new)
                                .push(feature.to_string());
                        }
                    } else {
                        // 这是主要特性
                        main_features.push(value.clone());
                    }
                }

                // 找到主要特性在所有特性中的索引
                let main_selected_indices: Vec<usize> = main_features
                    .iter()
                    .filter_map(|value| all_features.iter().position(|f| f == value))
                    .collect();

                // 获取依赖项信息
                let dependencies = get_package_dependencies(&metadata, package, deps_filter);

                // 将已选择的依赖项特性转换为索引
                let mut dep_selected_indices = HashMap::new();
                for dep in &dependencies {
                    if let Some(selected_features) = dep_selected_features.get(&dep.name) {
                        let indices: Vec<usize> = selected_features
                            .iter()
                            .filter_map(|feature| dep.features.iter().position(|f| f == feature))
                            .collect();
                        if !indices.is_empty() {
                            dep_selected_indices.insert(dep.name.clone(), indices);
                        }
                    }
                }

                // 创建ExtendedMultiSelectItem用于显示扩展多选界面
                let extended_multi_select_item = ExtendedMultiSelectItem {
                    variants: all_features,
                    selected_indices: main_selected_indices,
                    dependencies,
                    dep_selected_features: dep_selected_indices,
                };

                // 显示扩展多选对话框
                show_extended_multi_select(
                    s,
                    &format!("Features for {}", package),
                    &extended_multi_select_item,
                );
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

/// 获取包的依赖项信息，支持过滤
///
/// # 参数
/// - `metadata`: Cargo 元数据
/// - `package_name`: 要查询的包名
/// - `deps_filter`: 依赖项过滤器
///   - `Some(&[String])`: 只返回指定的依赖项
///   - `None`: 返回所有有 features 的依赖项
///
/// # 返回
/// 返回 `DepItem` 列表，每个包含依赖项名和其 features
/// 只包含有 features 的依赖项，按名称排序
fn get_package_dependencies(
    metadata: &cargo_metadata::Metadata,
    package_name: &str,
    deps_filter: Option<&[String]>,
) -> Vec<DepItem> {
    let mut dependencies = Vec::new();

    // 查找当前包
    if let Some(current_pkg) = metadata.packages.iter().find(|p| p.name == package_name) {
        // 遍历当前包的依赖项
        for dep in &current_pkg.dependencies {
            // 检查是否应该包含这个依赖项
            let should_include = match deps_filter {
                Some(filter_list) => filter_list.contains(&dep.name),
                None => true, // 如果没有过滤器，包含所有依赖项
            };

            if should_include {
                // 查找依赖项包的详细信息
                if let Some(dep_pkg) = metadata.packages.iter().find(|p| p.name == dep.name) {
                    let dep_features: Vec<String> = dep_pkg.features.keys().cloned().collect();

                    // 只添加有features的依赖项
                    if !dep_features.is_empty() {
                        dependencies.push(DepItem {
                            name: dep.name.clone(),
                            features: dep_features,
                        });
                    }
                }
            }
        }
    }

    // 按名称排序
    dependencies.sort_by(|a, b| a.name.cmp(&b.name));
    dependencies
}
