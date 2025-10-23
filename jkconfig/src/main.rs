#[macro_use]
extern crate log;

use clap::{Arg, Command};
use cursive::{Cursive, CursiveExt, views::Dialog};

use jkconfig::{data::AppData, ui::components::menu::menu_view};

// mod menu_view;
// use menu_view::MenuView;

/// 主函数
fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let matches = Command::new("jkconfig")
        .author("周睿 <zrufo747@outlook.com>")
        .arg(
            Arg::new("config")
                .long("config")
                .short('c')
                .value_name("FILE")
                .help("指定初始配置文件路径")
                .default_value(".project.toml"),
        )
        .arg(
            Arg::new("schema")
                .long("schema")
                .short('s')
                .value_name("FILE")
                .help("指定schema文件路径（默认基于配置文件名推导）"),
        )
        .get_matches();

    // 提取命令行参数
    let config_file = matches.get_one::<String>("config").map(|s| s.as_str());
    let schema_file = matches.get_one::<String>("schema").map(|s| s.as_str());

    // 初始化AppData
    let app_data = AppData::new(config_file, schema_file)?;
    let title = app_data.root.title.clone();
    let fields = app_data
        .root
        .menu()
        .children
        .values()
        .cloned()
        .collect::<Vec<_>>();

    // 创建Cursive应用
    let mut siv = Cursive::default();

    // 设置AppData为user_data
    siv.set_user_data(app_data);

    // 添加全局键盘事件处理
    siv.add_global_callback('q', handle_quit);
    siv.add_global_callback('Q', handle_quit);
    siv.add_global_callback('s', handle_save);
    siv.add_global_callback('S', handle_save);

    siv.add_fullscreen_layer(menu_view(&title, fields));

    // 运行应用
    siv.run();

    Ok(())
}

/// 处理退出 - Q键
fn handle_quit(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::text("Quit without saving?")
            .title("Quit")
            .button("Quit", |s| {
                s.quit();
            })
            .button("Back", |s| {
                s.pop_layer();
            }),
    );
}

/// 处理保存 - S键
fn handle_save(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::text("Save and exit?")
            .title("Save")
            .button("Ok", |s| {
                let app = s.user_data::<AppData>().unwrap();
                app.needs_save = true;
                if let Err(e) = app.on_exit() {
                    error!("Failed to save config: {}", e);
                }
            })
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}
