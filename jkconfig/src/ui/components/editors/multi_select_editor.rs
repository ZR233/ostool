use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextView},
};
use log::info;

use crate::{
    data::{app_data::AppData, item::ItemType, types::ElementType},
    ui::handle_back,
};

/// 多选项结构体
#[derive(Debug, Clone)]
pub struct MultiSelectItem {
    pub variants: Vec<String>,
    pub selected_indices: Vec<usize>,
}

/// 显示多选对话框
pub fn show_multi_select(s: &mut Cursive, title: &str, multi_select: &MultiSelectItem) {
    let mut select = SelectView::new();

    // 添加所有选项到SelectView
    for (idx, variant) in multi_select.variants.iter().enumerate() {
        let label = if multi_select.selected_indices.contains(&idx) {
            format!("[*] {}", variant) // 已选中
        } else {
            format!("[ ] {}", variant) // 未选中
        };
        select.add_item(label, idx);
    }

    // 保存完整的选项列表到应用数据中，供后续toggle_selection使用
    // 初始化多选择数据并保存到临时存储
    if let Some(app) = s.user_data::<AppData>() {
        // 获取当前正在编辑的项的key
        let current_key = if let Some(ElementType::Item(item)) = app.current() {
            item.base.key().to_string()
        } else {
            "unknown_key".to_string()
        };
        
        let data = (
            multi_select.selected_indices.clone(),
            multi_select.variants.clone(),
            current_key.clone(), // 保存当前项的key
        );
        app.temp_data = Some((
            current_key, // 使用当前项的key作为临时数据的key
            serde_json::to_value(data).unwrap(),
        ));
    }

    // 移除冗余的保存逻辑，因为已经在上面保存了完整数据

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!("Select Multiple: {}", title)))
                    .child(
                        TextView::new("(Press Enter to toggle selection)")
                            .style(cursive::theme::ColorStyle::secondary()),
                    )
                    .child(DummyView)
                    .child(ScrollView::new(select.with_name("multi_select")).fixed_height(20)),
            )
            .title("Features")
            .button("OK", on_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, toggle_selection),
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

        if let Some(app) = s.user_data::<AppData>() {
            if let Some((_, temp_value)) = &app.temp_data {
                // 尝试从temp_data中获取保存的(indices, variants, current_key)元组
                if let Ok(data) = 
                    serde_json::from_value::<(Vec<usize>, Vec<String>, String)>(temp_value.clone())
                {
                    selected_indices = data.0;
                    variants = data.1;
                    current_key = data.2;
                }
            }
        }

        // 切换选中状态
        if let Some(pos) = selected_indices.iter().position(|&x| x == current_selected_idx) {
            selected_indices.remove(pos); // 移除选中
        } else {
            selected_indices.push(current_selected_idx); // 添加选中
            selected_indices.sort(); // 保持有序
        }

        // 更新保存的数据
        if let Some(app) = s.user_data::<AppData>() {
            app.temp_data = Some((
                current_key.clone(),
                serde_json::to_value((selected_indices.clone(), variants.clone(), current_key)).unwrap(),
            ));
        }

        // 更新UI显示
        s.call_on_name("multi_select", |view: &mut SelectView<usize>| {
            view.clear();

            // 重新添加所有项，更新选中状态
            for (idx, variant) in variants.iter().enumerate() {
                let label = if selected_indices.contains(&idx) {
                    format!("[*] {}", variant)
                } else {
                    format!("[ ] {}", variant)
                };
                view.add_item(label, idx);
            }
            
            // 恢复原来的选择位置
            view.set_selection(current_selected_idx);
        });
    }
}

/// 确认选择
fn on_ok(s: &mut Cursive) {
    let app = s.user_data::<AppData>().unwrap();

    // 获取保存的多选择数据
    if let Some((_, temp_value)) = app.temp_data.take() {
        // 尝试解析保存的数据：(selected_indices, variants, current_key)
        if let Ok((selected_indices, variants, current_key)) = 
            serde_json::from_value::<(Vec<usize>, Vec<String>, String)>(temp_value)
        {
            // 根据索引获取选中的选项文本
            let selected_variants: Vec<String> = selected_indices
                .iter()
                .filter_map(|&idx| variants.get(idx).cloned())
                .collect();

            // 查找并更新对应的ArrayItem
            if let Some(ElementType::Item(item_mut)) = app.root.get_mut_by_key(&current_key) {
                if let ItemType::Array(array_mut) = &mut item_mut.item_type {
                    // 更新ArrayItem的values列表，只包含选中的选项
                    array_mut.values = selected_variants.clone();
                    app.needs_save = true;
                    info!(
                        "Multi select confirmed with {} items selected for key: {}",
                        selected_variants.len(),
                        current_key
                    );
                }
            } else {
                info!("Failed to find item with key: {}", current_key);
            }
        }
    }

    handle_back(s);
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
