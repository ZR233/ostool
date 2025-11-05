use cursive::{
    Cursive,
    event::{Event, EventResult, Key},
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, SelectView, TextView},
}; 
use log::info;

use crate::{
    data::{app_data::AppData},
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
        let data = (multi_select.selected_indices.clone(), multi_select.variants.clone());
        app.temp_data = Some((
            "multi_select_data".to_string(),
            serde_json::to_value(data).unwrap(),
        ));
    }

    // 移除冗余的保存逻辑，因为已经在上面保存了完整数据

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!("Select Multiple: {}", title)))
                    .child(TextView::new("(Press Space to toggle selection, Enter to confirm)")
                        .style(cursive::theme::ColorStyle::secondary()))
                    .child(DummyView)
                    .child(select.with_name("multi_select").fixed_height(10)),
            )
            .title("Multi Selection")
            .button("OK", on_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, toggle_selection)
    );
}

/// 切换当前选中项的选择状态
fn toggle_selection(s: &mut Cursive) {
    // 获取当前选中的项目
    let selection = s
        .call_on_name("multi_select", |v: &mut SelectView<usize>| v.selection())
        .unwrap();
    
    if let Some(selection_idx) = selection {
        // 获取保存的多选择数据
        let mut multi_select_data = (Vec::new(), Vec::new()); // 默认空数据
        
        if let Some(app) = s.user_data::<AppData>() {
            if let Some((_, temp_value)) = &app.temp_data {
                // 尝试从temp_data中获取保存的(indices, variants)元组
                if let Ok(data) = serde_json::from_value::<(Vec<usize>, Vec<String>)>(temp_value.clone()) {
                    multi_select_data = data;
                }
            }
        }
        
        let (mut selected_indices, variants) = multi_select_data;
        
        // 切换选中状态
        if let Some(pos) = selected_indices.iter().position(|&x| x == *selection_idx) {
            selected_indices.remove(pos); // 移除选中
        } else {
            selected_indices.push(*selection_idx); // 添加选中
            selected_indices.sort(); // 保持有序
        }
        
        // 更新保存的数据
        if let Some(app) = s.user_data::<AppData>() {
            app.temp_data = Some((
                "multi_select_data".to_string(),
                serde_json::to_value((selected_indices.clone(), variants.clone())).unwrap(),
            ));
        }
        
        // 更新UI显示
        s.call_on_name("multi_select", |view: &mut SelectView<usize>| {
            view.clear();
            
            // 重新添加所有项，更新选中状态
            for (idx, variant) in variants.iter().enumerate() {
                let label = if selected_indices.contains(&idx) {
                    format!("[x] {}", variant)
                } else {
                    format!("[ ] {}", variant)
                };
                view.add_item(label, idx);
            }
        });
    }
}

/// 确认选择
fn on_ok(s: &mut Cursive) {
    let app = s.user_data::<AppData>().unwrap();
    
    // 获取选中的索引列表
    let selected_indices = if let Some((_, temp_value)) = app.temp_data.take() {
        if let Ok(indices) = serde_json::from_value::<Vec<usize>>(temp_value) {
            indices
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 处理实际数据更新逻辑
    // 注意：实际使用时需要有机制来存储和获取所有选项的原始值列表
    // 这里的实现需要与show_multi_select函数配合使用，确保能正确获取选项文本
    info!("Multi select confirmed with {} items selected", selected_indices.len());
    
    handle_back(s);
}

/// 从ArrayItem创建MultiSelectItem
pub fn create_multi_select_from_array_item(array_item: &crate::data::item::ArrayItem, all_variants: &[String]) -> MultiSelectItem {
    // 找到已选中的索引
    let selected_indices = all_variants
        .iter()
        .enumerate()
        .filter(|(_, variant)| array_item.values.contains(variant))
        .map(|(idx, _)| idx)
        .collect();
    
    MultiSelectItem {
        variants: all_variants.to_vec(),
        selected_indices,
    }
}