use crate::data::types::ElementBase;

#[derive(Debug, Clone)]
pub struct Item {
    pub base: ElementBase,
    pub item_type: ItemType,
}

#[derive(Debug, Clone)]
pub enum ItemType {
    String {
        value: Option<String>,
        default: Option<String>,
    },
    Number {
        value: Option<f64>,
        default: Option<f64>,
    },
    Integer {
        value: Option<i64>,
        default: Option<i64>,
    },
    Boolean {
        value: bool,
        default: bool,
    },
    Enum(EnumItem),
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub variants: Vec<String>,
    pub value: Option<usize>,
    pub default: Option<usize>,
}

impl EnumItem {
    pub fn value_str(&self) -> Option<&str> {
        self.value
            .and_then(|idx| self.variants.get(idx).map(String::as_str))
    }
}
