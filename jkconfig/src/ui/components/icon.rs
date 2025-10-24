use crate::data::{item::ItemType, types::ElementType};

pub trait ItemDisplay {
    fn icon(&self) -> String;
    fn value(&self) -> String;
}

impl ItemDisplay for ElementType {
    fn icon(&self) -> String {
        if self.is_none() {
            if self.is_required {
                return " ❗ ".into();
            }

            return " 🔘 ".into();
        }

        let raw = match self {
            ElementType::Menu(_) => "📂",
            ElementType::OneOf(_) => "🔀",
            ElementType::Item(item) => match &item.item_type {
                ItemType::String { .. } => "🔡",
                ItemType::Number { .. } => "🔢",
                ItemType::Integer { .. } => "🔢",
                ItemType::Boolean { value, .. } => {
                    if *value {
                        "✅"
                    } else {
                        "❎"
                    }
                }
                ItemType::Enum(_) => "📚",
            },
        };
        if self.is_required {
            format!(" {raw} ")
        } else {
            format!("<{raw}>")
        }
    }

    fn value(&self) -> String {
        match self {
            ElementType::Menu(_) => String::new(),
            ElementType::OneOf(one_of) => {
                if let Some(selected) = one_of.selected() {
                    let name = match selected {
                        ElementType::Menu(menu) => menu.struct_name.clone(),
                        ElementType::Item(item) => match &item.item_type {
                            ItemType::Enum(enum_item) => {
                                enum_item.variants.first().cloned().unwrap_or_default()
                            }
                            _ => String::new(),
                        },
                        _ => String::new(),
                    };

                    format!("<{name}>")
                } else {
                    "<Unset>".to_string()
                }
            }
            ElementType::Item(item) => match &item.item_type {
                ItemType::String { value, .. } => value.clone().unwrap_or_default(),
                ItemType::Number { value, .. } => {
                    if let Some(v) = value {
                        v.to_string()
                    } else {
                        String::new()
                    }
                }
                ItemType::Integer { value, .. } => {
                    if let Some(v) = value {
                        v.to_string()
                    } else {
                        String::new()
                    }
                }
                ItemType::Boolean { .. } => String::new(),
                ItemType::Enum(enum_item) => {
                    if let Some(v) = enum_item.value_str() {
                        v.to_string()
                    } else {
                        String::new()
                    }
                }
            },
        }
    }
}
