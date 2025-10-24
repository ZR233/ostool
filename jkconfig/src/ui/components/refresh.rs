use cursive::{Cursive, views::SelectView};

use crate::data::{app_data::AppData, types::ElementType};

use super::menu::format_item_label;

/// 刷新当前菜单视图的内容
pub fn refresh_current_menu(s: &mut Cursive) {
    // 获取 AppData
    let fields = {
        let app_data = match s.user_data::<AppData>() {
            Some(data) => data,
            None => return,
        };

        // 获取当前路径的数据
        match app_data.current() {
            Some(ElementType::Menu(menu)) => menu.children.values().cloned().collect::<Vec<_>>(),
            _ => {
                // 如果当前路径不是菜单，则获取根菜单
                app_data
                    .root
                    .menu()
                    .children
                    .values()
                    .cloned()
                    .collect::<Vec<_>>()
            }
        }
    };

    // 更新 SelectView
    s.call_on_name("main_select", |view: &mut SelectView<ElementType>| {
        // 保存当前选中的索引
        let current_selection = view.selected_id();

        // 清空并重新填充列表
        view.clear();
        for field in fields {
            let label = format_item_label(&field);
            view.add_item(label, field);
        }

        // 恢复之前的选择位置
        if let Some(idx) = current_selection
            && idx < view.len()
        {
            view.set_selection(idx);
        }
    });
}
