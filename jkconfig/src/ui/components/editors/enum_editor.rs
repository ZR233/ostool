use cursive::{
    Cursive,
    event::Key,
    view::{Nameable, Resizable},
    views::{Dialog, DummyView, LinearLayout, OnEventView, SelectView, TextView},
};

use crate::{
    data::{
        item::{EnumItem, ItemType},
        types::ElementType,
    },
    ui::handle_back,
};

/// 显示枚举选择对话框
pub fn show_enum_select(s: &mut Cursive, title: &str, enum_item: &EnumItem) {
    let mut select = SelectView::new();

    for (idx, variant) in enum_item.variants.iter().enumerate() {
        let label = if Some(idx) == enum_item.value {
            format!("(*) {}", variant)
        } else {
            format!("( ) {}", variant)
        };
        select.add_item(label, idx);
    }

    s.add_layer(
        OnEventView::new(
            Dialog::around(
                LinearLayout::vertical()
                    .child(TextView::new(format!("Select: {}", title)))
                    .child(DummyView)
                    .child(select.with_name("enum_select").fixed_height(10)),
            )
            .title("Select Option")
            .button("OK", on_ok)
            .button("Cancel", handle_back),
        )
        .on_event(Key::Enter, on_ok),
    );
}

fn on_ok(s: &mut Cursive) {
    let selection = s
        .call_on_name("enum_select", |v: &mut SelectView<usize>| v.selection())
        .unwrap();
    let Some(selection) = selection else {
        return;
    };

    // // 先获取selected_variant的值
    // let selected_variant: String = {
    //     let app = s.user_data::<crate::data::app_data::AppData>().unwrap();
    //     if let Some(ElementType::Item(item)) = app.current() {
    //         if let ItemType::Enum(enum_item) = &item.item_type {
    //             if let Some(variant) = enum_item.variants.get(*selection) {
    //                 variant.clone()
    //             } else {
    //                 return;
    //             }
    //         } else {
    //             return;
    //         }
    //     } else {
    //         return;
    //     }
    // };
    
    // // 再次获取mut引用进行修改
    // let mut app = s.user_data::<crate::data::app_data::AppData>().unwrap();
    
    // // 处理temp_data情况（针对ArrayItem转换为EnumItem的场景）
    // if let Some((key, _temp_value)) = app.temp_data.take() {
    //     // 获取key对应的ArrayItem并更新
    //     if let Some(ElementType::Item(item_mut)) = app.root.get_mut_by_key(&key) {
    //         if let ItemType::Array(array_mut) = &mut item_mut.item_type {
    //             // 添加选中的feature到数组（如果不存在）
    //             if !array_mut.values.contains(&selected_variant) {
    //                 array_mut.values.push(selected_variant.clone());
    //                 app.needs_save = true;
    //                 info!("Added feature: {} to array", selected_variant);
    //             }
    //         }
    //     }
    // } else {
    //     // 常规Enum类型处理
    //     if let Some(ElementType::Item(item)) = app.current_mut() {
    //         if let ItemType::Enum(en) = &mut item.item_type {
    //             en.value = Some(*selection);
    //             app.needs_save = true;
    //         }
    //     }
    // }
    
    handle_back(s);
}
