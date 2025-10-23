use cursive::{
    view::{IntoBoxedView, Resizable},
    views::{LinearLayout, SelectView, TextView},
};

use crate::data::types::ElementType;

pub fn menu_view(title: &str, fields: Vec<ElementType>) -> impl IntoBoxedView {
    let mut l = SelectView::new();

    for field in fields {
        l.add_item(field.title.clone(), field);
    }

    LinearLayout::vertical()
        .child(TextView::new(title).center())
        .child(l.on_submit(on_submit).full_width().full_height())
}

fn on_submit(s: &mut cursive::Cursive, item: &ElementType) {
    match item {
        ElementType::Menu(menu) => {
            let title = menu.title.clone();
            let fields = menu.children.values().cloned().collect();
            s.add_fullscreen_layer(menu_view(&title, fields));
        }
        ElementType::OneOf(one_of) => {}
        ElementType::Item(item) => match &item.item_type {
            crate::data::item::ItemType::String { value, default } => todo!(),
            crate::data::item::ItemType::Number { value, default } => todo!(),
            crate::data::item::ItemType::Integer { value, default } => todo!(),
            crate::data::item::ItemType::Boolean { value, default } => todo!(),
            crate::data::item::ItemType::Enum(enum_item) => todo!(),
        },
    }
}
