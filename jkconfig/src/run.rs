use std::{fmt::format, path::Path};

use anyhow::Context;
pub use cursive;
use cursive::{Cursive, CursiveExt, event::Key};
use schemars::JsonSchema;

use crate::{
    data::{AppData, app_data::default_schema_by_init},
    ui::{components::menu::menu_view, handle_back, handle_quit, handle_save},
};

pub use crate::data::app_data::ElemHock;

pub async fn prepare_config<'a, C: JsonSchema + serde::Deserialize<'a>>(
    config_path: impl AsRef<Path>,
    content: &'a mut String,
    elem_hocks: Vec<ElemHock>,
) -> anyhow::Result<C> {
    let config_path = config_path.as_ref();
    let schema = schemars::schema_for!(C);
    let schema_json = serde_json::to_value(&schema)?;

    *content = tokio::fs::read_to_string(&config_path)
        .await
        .with_context(|| format!("Failed to read {}", config_path.display()))?;

    let ext = config_path
        .extension()
        .map(|s| format!("{}", s.display()))
        .unwrap_or(String::new());

    if let Ok(c) = to_typed::<C>(content, &ext) {
        return Ok(c);
    }

    *content = get_content_by_ui(config_path, &schema_json, elem_hocks).await?;

    let config = match ext.as_str() {
        "json" => serde_json::from_str::<C>(content)?,
        "toml" => toml::from_str::<C>(content)?,
        _ => {
            anyhow::bail!(
                "unsupported config file extension: {}",
                config_path.display()
            );
        }
    };

    Ok(config)
}

fn to_typed<'a, C: JsonSchema + serde::Deserialize<'a>>(
    s: &'a str,
    ext: &str,
) -> anyhow::Result<C> {
    let c = match ext {
        "json" => serde_json::from_str::<C>(s)?,
        "toml" => toml::from_str::<C>(s)?,
        _ => {
            anyhow::bail!("unsupported config file extension: {ext}",);
        }
    };
    Ok(c)
}

async fn get_content_by_ui(
    config: impl AsRef<Path>,
    schema: &serde_json::Value,
    elem_hocks: Vec<ElemHock>,
) -> anyhow::Result<String> {
    let mut app_data = AppData::new_with_schema(Some(config), schema)?;
    app_data.elem_hocks = elem_hocks;

    let title = app_data.root.title.clone();
    let fields = app_data.root.menu().fields();

    #[cfg(feature = "logging")]
    {
        cursive::logger::init();
        cursive::logger::set_filter_levels_from_env();
    }
    // 创建Cursive应用
    let mut siv = Cursive::default();

    // 设置AppData为user_data
    siv.set_user_data(app_data);

    // 添加全局键盘事件处理
    siv.add_global_callback('q', handle_quit);
    siv.add_global_callback('Q', handle_quit);
    siv.add_global_callback('s', handle_save);
    siv.add_global_callback('S', handle_save);
    siv.add_global_callback(Key::Esc, handle_back);
    siv.add_global_callback('~', cursive::Cursive::toggle_debug_console);
    // 初始菜单路径为空
    siv.add_fullscreen_layer(menu_view(&title, "", fields));

    // 运行应用
    siv.run();

    let app = siv.take_user_data::<AppData>().unwrap();
    // println!("Data: \n{:#?}", app.root);
    Ok(app.root.as_json().to_string())
}

pub fn run_type<T: JsonSchema>(
    config: Option<impl AsRef<Path>>,
    elem_hocks: Vec<ElemHock>,
) -> anyhow::Result<AppData> {
    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_value(&schema)?;
    run(config, &schema_json, elem_hocks)
}

pub fn run(
    config: Option<impl AsRef<Path>>,
    schema: &serde_json::Value,
    elem_hocks: Vec<ElemHock>,
) -> anyhow::Result<AppData> {
    let mut app_data = AppData::new_with_schema(config, schema)?;
    app_data.elem_hocks = elem_hocks;

    let title = app_data.root.title.clone();
    let fields = app_data.root.menu().fields();

    cursive::logger::init();
    cursive::logger::set_filter_levels_from_env();
    // 创建Cursive应用
    let mut siv = Cursive::default();

    // 设置AppData为user_data
    siv.set_user_data(app_data);

    // 添加全局键盘事件处理
    siv.add_global_callback('q', handle_quit);
    siv.add_global_callback('Q', handle_quit);
    siv.add_global_callback('s', handle_save);
    siv.add_global_callback('S', handle_save);
    siv.add_global_callback(Key::Esc, handle_back);
    siv.add_global_callback('~', cursive::Cursive::toggle_debug_console);
    // 初始菜单路径为空
    siv.add_fullscreen_layer(menu_view(&title, "", fields));

    // 运行应用
    siv.run();

    let mut app = siv.take_user_data::<AppData>().unwrap();
    // println!("Data: \n{:#?}", app.root);
    app.on_exit()?;

    Ok(app)
}
