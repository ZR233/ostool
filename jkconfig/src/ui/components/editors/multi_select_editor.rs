use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextView},
};

use crate::data::{app_data::AppData, item::ItemType, types::ElementType};

/// 多选项结构体
#[derive(Debug, Clone)]
pub struct MultiSelectItem {
    pub variants: Vec<String>,
    pub selected_indices: Vec<usize>,
}

/// 依赖项信息结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepItem {
    pub name: String,
    pub features: Vec<String>,
}

/// 扩展的多选项结构体，支持依赖项
#[derive(Debug, Clone)]
pub struct ExtendedMultiSelectItem {
    pub variants: Vec<String>,
    pub selected_indices: Vec<usize>,
    pub dependencies: Vec<DepItem>,
    pub dep_selected_features: HashMap<String, Vec<usize>>, // dep_name -> selected feature indices
}

use std::collections::HashMap;

/// 多选临时数据结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiSelectTempData {
    pub selected_indices: Vec<usize>,
    pub variants: Vec<String>,
    pub current_key: String,
}

/// 扩展多选临时数据结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtendedMultiSelectTempData {
    pub selected_indices: Vec<usize>,
    pub variants: Vec<String>,
    pub dependencies: Vec<DepItem>,
    pub dep_selected_features: HashMap<String, Vec<usize>>,
    pub current_key: String,
}

/// 依赖项特性选择临时数据结构体
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DepFeaturesTempData {
    pub main_selected_indices: Vec<usize>,
    pub main_variants: Vec<String>,
    pub dep_name: String,
    pub dep_features: Vec<String>,
    pub selected_indices: Vec<usize>,
    pub current_key: String,
}

/// 显示多选全屏界面
pub fn show_multi_select(s: &mut Cursive, title: &str, multi_select: &MultiSelectItem) {
    let mut select = SelectView::new();

    // 添加所有选项到SelectView，使用更美观的标记
    for (idx, variant) in multi_select.variants.iter().enumerate() {
        let label = if multi_select.selected_indices.contains(&idx) {
            format!("✓ {}  [已选择]", variant) // 已选中 - 使用对勾符号
        } else {
            format!("○ {}  [未选择]", variant) // 未选中 - 使用圆圈符号
        };
        select.add_item(label, idx);
    }

    // 保存完整的选项列表到应用数据中，供后续toggle_selection使用
    if let Some(app) = s.user_data::<AppData>() {
        // 获取当前正在编辑的项的key
        let current_key = if let Some(ElementType::Item(item)) = app.current() {
            item.base.key().to_string()
        } else {
            "unknown_key".to_string()
        };

        let data = MultiSelectTempData {
            selected_indices: multi_select.selected_indices.clone(),
            variants: multi_select.variants.clone(),
            current_key: current_key.clone(),
        };
        app.temp_data = Some((current_key, serde_json::to_value(data).unwrap()));
    }

    // 创建标题样式
    let title_view =
        TextView::new(format!("📋 {}", title)).style(cursive::theme::ColorStyle::title_primary());

    // 创建状态栏
    let status_text = TextView::new(format!(
        "已选择 {} / {} 项 | Enter: 切换选择 | ESC: 退出",
        multi_select.selected_indices.len(),
        multi_select.variants.len()
    ))
    .style(cursive::theme::ColorStyle::secondary())
    .with_name("status_text");

    // 创建全屏布局
    let main_layout = LinearLayout::vertical()
        .child(title_view)
        .child(DummyView)
        .child(status_text)
        .child(DummyView)
        .child(
            ScrollView::new(select.with_name("multi_select"))
                .fixed_height(20) // 设置适当的高度
                .full_width(),
        )
        .child(DummyView);

    // 创建提示文本
    let hint_text = TextView::new("💡 提示: 选择后自动保存，无需确认")
        .style(cursive::theme::ColorStyle::tertiary());

    // 创建全屏对话框容器
    let fullscreen_dialog = cursive::views::Panel::new(
        LinearLayout::vertical()
            .child(main_layout.full_height())
            .child(DummyView)
            .child(hint_text)
            .child(DummyView),
    )
    .title("🌟 多选界面");

    // 添加全屏层
    s.add_fullscreen_layer(
        OnEventView::new(fullscreen_dialog)
            .on_event(Key::Enter, toggle_selection)
            .on_event(' ', toggle_selection) // 添加空格键支持
            .on_event(Key::Right, |s| {
                s.on_event(cursive::event::Event::Key(cursive::event::Key::Tab));
            }),
    );
}

/// 切换当前选中项的选择状态
fn toggle_selection(s: &mut Cursive) {
    // 获取当前选中的项目
    let selection = s
        .call_on_name("multi_select", |v: &mut SelectView<usize>| v.selection())
        .unwrap();

    if let Some(selection_idx) = selection {
        // 保存当前选中的索引值
        let current_selected_idx = *selection_idx;

        // 获取保存的多选择数据
        let mut selected_indices = Vec::new();
        let mut variants = Vec::new();
        let mut current_key = String::new();

        if let Some(app) = s.user_data::<AppData>()
            && let Some((_, temp_value)) = &app.temp_data
        {
            // 尝试从temp_data中获取保存的结构体数据
            if let Ok(data) = serde_json::from_value::<MultiSelectTempData>(temp_value.clone()) {
                selected_indices = data.selected_indices;
                variants = data.variants;
                current_key = data.current_key;
            }
        }

        // 切换选中状态
        if let Some(pos) = selected_indices
            .iter()
            .position(|&x| x == current_selected_idx)
        {
            selected_indices.remove(pos); // 移除选中
        } else {
            selected_indices.push(current_selected_idx); // 添加选中
            selected_indices.sort(); // 保持有序
        }

        // 更新保存的数据
        if let Some(app) = s.user_data::<AppData>() {
            let data = MultiSelectTempData {
                selected_indices: selected_indices.clone(),
                variants: variants.clone(),
                current_key: current_key.clone(),
            };
            app.temp_data = Some((current_key.clone(), serde_json::to_value(data).unwrap()));

            // 直接更新数据到对应的ArrayItem
            let selected_variants: Vec<String> = selected_indices
                .iter()
                .filter_map(|&idx| variants.get(idx).cloned())
                .collect();

            if let Some(ElementType::Item(item_mut)) = app.root.get_mut_by_key(&current_key)
                && let ItemType::Array(array_mut) = &mut item_mut.item_type
            {
                array_mut.values = selected_variants.clone();
                app.needs_save = true;
                info!(
                    "Multi select updated with {} items selected for key: {}",
                    selected_variants.len(),
                    current_key
                );
            }
        }

        // 更新UI显示
        s.call_on_name("multi_select", |view: &mut SelectView<usize>| {
            view.clear();

            // 重新添加所有项，更新选中状态（使用新的美观标记）
            for (idx, variant) in variants.iter().enumerate() {
                let label = if selected_indices.contains(&idx) {
                    format!("✓ {}  [已选择]", variant) // 已选中 - 使用对勾符号
                } else {
                    format!("○ {}  [未选择]", variant) // 未选中 - 使用圆圈符号
                };
                view.add_item(label, idx);
            }

            // 恢复原来的选择位置
            view.set_selection(current_selected_idx);
        });

        // 更新状态栏显示
        s.call_on_name("status_text", |view: &mut TextView| {
            view.set_content(format!(
                "已选择 {} / {} 项 | Enter: 切换选择 | ESC: 退出",
                selected_indices.len(),
                variants.len()
            ));
        });
    }
}

/// 从ArrayItem创建MultiSelectItem
pub fn create_multi_select_from_array_item(
    array_item: &crate::data::item::ArrayItem,
    all_variants: &[String],
) -> MultiSelectItem {
    // 创建新的已保存选项集合，只保留那些在新获取选项列表中存在的选项
    let valid_saved_values: Vec<String> = array_item
        .values
        .iter()
        .filter(|&saved_val| all_variants.contains(saved_val))
        .cloned()
        .collect();

    // 找到这些有效保存选项在新获取列表中的索引
    let selected_indices: Vec<usize> = all_variants
        .iter()
        .enumerate()
        .filter(|(_, variant)| valid_saved_values.contains(variant))
        .map(|(idx, _)| idx)
        .collect();

    MultiSelectItem {
        variants: all_variants.to_vec(),
        selected_indices,
    }
}

/// 显示扩展的多选全屏界面，支持依赖项选择
pub fn show_extended_multi_select(
    s: &mut Cursive,
    title: &str,
    extended_multi_select: &ExtendedMultiSelectItem,
) {
    let mut select = SelectView::new();

    // 添加主要特性选项
    for (idx, variant) in extended_multi_select.variants.iter().enumerate() {
        let label = if extended_multi_select.selected_indices.contains(&idx) {
            format!("✓ {}  [已选择]", variant)
        } else {
            format!("○ {}  [未选择]", variant)
        };
        select.add_item(label, idx);
    }

    // 添加分隔符
    select.add_item("--- 依赖项 Features ---".to_string(), usize::MAX);

    // 添加依赖项选项，使用唯一索引
    for (dep_idx, dep) in extended_multi_select.dependencies.iter().enumerate() {
        let selected_count = extended_multi_select
            .dep_selected_features
            .get(&dep.name)
            .map(|indices| indices.len())
            .unwrap_or(0);

        let label = if selected_count > 0 {
            format!("📦 {} ({} features selected)", dep.name, selected_count)
        } else {
            format!("📦 {} (no features selected)", dep.name)
        };
        // 使用 variants.len() + 1 + dep_idx 作为唯一索引
        let unique_dep_index = extended_multi_select.variants.len() + 1 + dep_idx;
        select.add_item(label, unique_dep_index);
    }

    // 保存数据到应用数据中
    if let Some(app) = s.user_data::<AppData>() {
        let current_key = if let Some(ElementType::Item(item)) = app.current() {
            item.base.key().to_string()
        } else {
            "unknown_key".to_string()
        };

        let data = ExtendedMultiSelectTempData {
            selected_indices: extended_multi_select.selected_indices.clone(),
            variants: extended_multi_select.variants.clone(),
            dependencies: extended_multi_select.dependencies.clone(),
            dep_selected_features: extended_multi_select.dep_selected_features.clone(),
            current_key: current_key.clone(),
        };
        app.temp_data = Some((current_key, serde_json::to_value(data).unwrap()));
    }

    // 创建标题样式
    let title_view =
        TextView::new(format!("📋 {}", title)).style(cursive::theme::ColorStyle::title_primary());

    // 创建状态栏
    let status_text = TextView::new(format!(
        "已选择 {} / {} 项 | Enter: 切换选择/进入依赖项 | ESC: 退出",
        extended_multi_select.selected_indices.len(),
        extended_multi_select.variants.len()
    ))
    .style(cursive::theme::ColorStyle::secondary())
    .with_name("extended_status_text");

    // 创建全屏布局
    let main_layout = LinearLayout::vertical()
        .child(title_view)
        .child(DummyView)
        .child(status_text)
        .child(DummyView)
        .child(
            ScrollView::new(select.with_name("extended_multi_select"))
                .fixed_height(20)
                .full_width(),
        )
        .child(DummyView);

    // 创建提示文本
    let hint_text = TextView::new("💡 提示: 选择后自动保存，进入依赖项选择后也会自动更新")
        .style(cursive::theme::ColorStyle::tertiary());

    // 创建全屏对话框容器
    let fullscreen_dialog = cursive::views::Panel::new(
        LinearLayout::vertical()
            .child(main_layout.full_height())
            .child(DummyView)
            .child(hint_text)
            .child(DummyView),
    )
    .title("🌟 特性与依赖项选择");

    // 添加全屏层
    s.add_fullscreen_layer(
        OnEventView::new(fullscreen_dialog)
            .on_event(Key::Enter, toggle_extended_selection)
            .on_event(' ', toggle_extended_selection)
            .on_event(Key::Right, |s| {
                s.on_event(cursive::event::Event::Key(cursive::event::Key::Tab));
            }),
    );
}

/// 切换扩展选择状态或进入依赖项选择
fn toggle_extended_selection(s: &mut Cursive) {
    let selection = s
        .call_on_name("extended_multi_select", |v: &mut SelectView<usize>| {
            v.selection()
        })
        .unwrap();

    if let Some(selection_idx) = selection {
        let current_selected_idx = *selection_idx;

        // 获取保存的数据
        let mut selected_indices = Vec::new();
        let mut variants = Vec::new();
        let mut dependencies = Vec::new();
        let mut dep_selected_features = HashMap::new();
        let mut current_key = String::new();

        if let Some(app) = s.user_data::<AppData>()
            && let Some((_, temp_value)) = &app.temp_data
            && let Ok(data) =
                serde_json::from_value::<ExtendedMultiSelectTempData>(temp_value.clone())
        {
            selected_indices = data.selected_indices;
            variants = data.variants;
            dependencies = data.dependencies;
            dep_selected_features = data.dep_selected_features;
            current_key = data.current_key;
        }

        // 检查是否点击了依赖项
        if current_selected_idx > variants.len() && current_selected_idx != usize::MAX {
            // 这是依赖项，计算依赖项索引
            let dep_index = current_selected_idx - variants.len() - 1; // 减1是因为分隔符
            if let Some(dep) = dependencies.get(dep_index) {
                // 显示依赖项的features选择
                show_dep_features_select(
                    s,
                    dep,
                    &dep_selected_features,
                    &selected_indices,
                    &variants,
                    &current_key,
                );
                return;
            }
        }

        // 切换主要特性选择状态
        if let Some(pos) = selected_indices
            .iter()
            .position(|&x| x == current_selected_idx)
        {
            selected_indices.remove(pos);
        } else {
            selected_indices.push(current_selected_idx);
            selected_indices.sort();
        }

        // 更新保存的数据并直接保存到ArrayItem
        if let Some(app) = s.user_data::<AppData>() {
            let data = ExtendedMultiSelectTempData {
                selected_indices: selected_indices.clone(),
                variants: variants.clone(),
                dependencies: dependencies.clone(),
                dep_selected_features: dep_selected_features.clone(),
                current_key: current_key.clone(),
            };
            app.temp_data = Some((current_key.clone(), serde_json::to_value(data).unwrap()));

            // 直接更新数据到对应的ArrayItem
            let selected_variants: Vec<String> = selected_indices
                .iter()
                .filter_map(|&idx| variants.get(idx).cloned())
                .collect();

            // 获取依赖项选中的features
            let mut dep_features: Vec<String> = Vec::new();
            for (dep_name, selected_feature_indices) in &dep_selected_features {
                if let Some(dep) = dependencies.iter().find(|d| d.name == *dep_name) {
                    for &feature_idx in selected_feature_indices {
                        if let Some(feature) = dep.features.get(feature_idx) {
                            dep_features.push(format!("{}/{}", dep_name, feature));
                        }
                    }
                }
            }

            // 合并所有选中的特性
            let all_selected: Vec<String> =
                selected_variants.into_iter().chain(dep_features).collect();

            if let Some(ElementType::Item(item_mut)) = app.root.get_mut_by_key(&current_key)
                && let ItemType::Array(array_mut) = &mut item_mut.item_type
            {
                array_mut.values = all_selected.clone();
                app.needs_save = true;
                info!(
                    "Extended multi select updated with {} items selected for key: {}",
                    all_selected.len(),
                    current_key
                );
            }
        }

        // 更新UI显示
        s.call_on_name("extended_multi_select", |view: &mut SelectView<usize>| {
            view.clear();

            // 重新添加主要特性
            for (idx, variant) in variants.iter().enumerate() {
                let label = if selected_indices.contains(&idx) {
                    format!("✓ {}  [已选择]", variant)
                } else {
                    format!("○ {}  [未选择]", variant)
                };
                view.add_item(label, idx);
            }

            // 添加分隔符
            view.add_item("--- 依赖项 Features ---".to_string(), usize::MAX);

            // 重新添加依赖项，使用唯一索引
            for (dep_idx, dep) in dependencies.iter().enumerate() {
                let selected_count = dep_selected_features
                    .get(&dep.name)
                    .map(|indices| indices.len())
                    .unwrap_or(0);

                let label = if selected_count > 0 {
                    format!("📦 {} ({} features selected)", dep.name, selected_count)
                } else {
                    format!("📦 {} (no features selected)", dep.name)
                };
                // 使用 variants.len() + 1 + dep_idx 作为唯一索引
                let unique_dep_index = variants.len() + 1 + dep_idx;
                view.add_item(label, unique_dep_index);
            }

            view.set_selection(current_selected_idx);
        });

        // 更新状态栏显示
        s.call_on_name("extended_status_text", |view: &mut TextView| {
            view.set_content(format!(
                "已选择 {} / {} 项 | Enter: 切换选择/进入依赖项 | ESC: 退出",
                selected_indices.len(),
                variants.len()
            ));
        });
    }
}

/// 显示依赖项的features选择
fn show_dep_features_select(
    s: &mut Cursive,
    dep: &DepItem,
    dep_selected_features: &HashMap<String, Vec<usize>>,
    main_selected_indices: &[usize],
    main_variants: &[String],
    current_key: &str,
) {
    let mut select = SelectView::new();

    let selected_indices = dep_selected_features
        .get(&dep.name)
        .cloned()
        .unwrap_or_default();
    let selected_count = selected_indices.len();

    // 添加依赖项的features
    for (idx, feature) in dep.features.iter().enumerate() {
        let label = if selected_indices.contains(&idx) {
            format!("✓ {}  [已选择]", feature)
        } else {
            format!("○ {}  [未选择]", feature)
        };
        select.add_item(label, idx);
    }

    // 保存依赖项选择数据
    if let Some(app) = s.user_data::<AppData>() {
        let data = DepFeaturesTempData {
            main_selected_indices: main_selected_indices.to_vec(),
            main_variants: main_variants.to_vec(),
            dep_name: dep.name.clone(),
            dep_features: dep.features.clone(),
            selected_indices,
            current_key: current_key.to_string(),
        };
        app.temp_data = Some((
            "dep_features_select".to_string(),
            serde_json::to_value(data).unwrap(),
        ));
    }

    // 创建标题
    let title_view = TextView::new(format!("📦 {} Features", dep.name))
        .style(cursive::theme::ColorStyle::title_primary());

    // 创建状态栏
    let status_text = TextView::new(format!(
        "已选择 {} / {} 项 | Enter: 切换选择 | ESC: 返回",
        selected_count,
        dep.features.len()
    ))
    .style(cursive::theme::ColorStyle::secondary())
    .with_name("dep_status_text");

    // 创建布局
    let main_layout = LinearLayout::vertical()
        .child(title_view)
        .child(DummyView)
        .child(status_text)
        .child(DummyView)
        .child(
            ScrollView::new(select.with_name("dep_features_select"))
                .fixed_height(20)
                .full_width(),
        )
        .child(DummyView);

    // 创建提示文本
    let hint_text = TextView::new("💡 提示: 选择后自动更新到主界面，并保存")
        .style(cursive::theme::ColorStyle::tertiary());

    // 创建对话框
    let dialog = cursive::views::Panel::new(
        LinearLayout::vertical()
            .child(main_layout)
            .child(DummyView)
            .child(hint_text)
            .child(DummyView),
    )
    .title("🌟 依赖项特性选择");

    s.add_fullscreen_layer(
        OnEventView::new(dialog)
            .on_event(Key::Enter, toggle_dep_features_selection)
            .on_event(' ', toggle_dep_features_selection),
    );
}

/// 切换依赖项feature选择
fn toggle_dep_features_selection(s: &mut Cursive) {
    let selection = s
        .call_on_name("dep_features_select", |v: &mut SelectView<usize>| {
            v.selection()
        })
        .unwrap();

    if let Some(selection_idx) = selection {
        let current_selected_idx = *selection_idx;

        // 获取保存的数据
        let mut main_selected_indices = Vec::new();
        let mut main_variants = Vec::new();
        let mut dep_name = String::new();
        let mut dep_features = Vec::new();
        let mut selected_indices = Vec::new();
        let mut current_key = String::new();

        if let Some(app) = s.user_data::<AppData>()
            && let Some((key, temp_value)) = &app.temp_data
            && key == "dep_features_select"
            && let Ok(data) = serde_json::from_value::<DepFeaturesTempData>(temp_value.clone())
        {
            main_selected_indices = data.main_selected_indices;
            main_variants = data.main_variants;
            dep_name = data.dep_name;
            dep_features = data.dep_features;
            selected_indices = data.selected_indices;
            current_key = data.current_key;
        }

        // 切换选择状态
        if let Some(pos) = selected_indices
            .iter()
            .position(|&x| x == current_selected_idx)
        {
            selected_indices.remove(pos);
        } else {
            selected_indices.push(current_selected_idx);
            selected_indices.sort();
        }

        // 更新数据并更新主界面和ArrayItem
        if let Some(app) = s.user_data::<AppData>() {
            let data = DepFeaturesTempData {
                main_selected_indices: main_selected_indices.clone(),
                main_variants: main_variants.clone(),
                dep_name: dep_name.clone(),
                dep_features: dep_features.clone(),
                selected_indices: selected_indices.clone(),
                current_key: current_key.clone(),
            };
            app.temp_data = Some((
                "dep_features_select".to_string(),
                serde_json::to_value(data).unwrap(),
            ));

            // 创建扩展多选数据，包含当前依赖项的选择结果
            let mut dep_selected_features = HashMap::new();
            dep_selected_features.insert(dep_name.clone(), selected_indices.clone());

            // 获取主要选中的特性
            let selected_variants: Vec<String> = main_selected_indices
                .iter()
                .filter_map(|&idx| main_variants.get(idx).cloned())
                .collect();

            // 获取当前依赖项选中的features
            let mut dep_features_selected: Vec<String> = Vec::new();
            for &feature_idx in &selected_indices {
                if let Some(feature) = dep_features.get(feature_idx) {
                    dep_features_selected.push(format!("{}/{}", dep_name, feature));
                }
            }

            // 合并所有选中的特性
            let all_selected: Vec<String> = selected_variants
                .into_iter()
                .chain(dep_features_selected)
                .collect();

            if let Some(ElementType::Item(item_mut)) = app.root.get_mut_by_key(&current_key)
                && let ItemType::Array(array_mut) = &mut item_mut.item_type
            {
                array_mut.values = all_selected.clone();
                app.needs_save = true;
                info!(
                    "Dep features select updated with {} features for {} and total {} items for key: {}",
                    selected_indices.len(),
                    dep_name,
                    all_selected.len(),
                    current_key
                );
            }

            // 更新状态栏显示
            s.call_on_name("dep_status_text", |view: &mut TextView| {
                view.set_content(format!(
                    "已选择 {} / {} 项 | Enter: 切换选择 | ESC: 返回",
                    selected_indices.len(),
                    dep_features.len()
                ));
            });
        }

        // 更新UI
        s.call_on_name("dep_features_select", |view: &mut SelectView<usize>| {
            view.clear();

            for (idx, feature) in dep_features.iter().enumerate() {
                let label = if selected_indices.contains(&idx) {
                    format!("✓ {}  [已选择]", feature)
                } else {
                    format!("○ {}  [未选择]", feature)
                };
                view.add_item(label, idx);
            }

            view.set_selection(current_selected_idx);
        });
    }
}
