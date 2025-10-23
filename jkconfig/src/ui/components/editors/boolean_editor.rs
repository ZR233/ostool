use cursive::Cursive;
use cursive::views::Dialog;

use crate::ui::components::refresh::refresh_current_menu;

/// 切换布尔值
pub fn toggle_boolean(s: &mut Cursive, key: &str) {
    // TODO: 实现布尔值切换逻辑
    // 需要访问 AppData 并更新值
    s.add_layer(Dialog::info(format!("Toggle boolean: {}", key)));
    // 刷新菜单显示最新值
    refresh_current_menu(s);
}
