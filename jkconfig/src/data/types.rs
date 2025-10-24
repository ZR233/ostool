use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use crate::data::{item::Item, menu::Menu, oneof::OneOf, schema::SchemaError};

use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct ElementBase {
    pub path: PathBuf,
    pub title: String,
    pub help: Option<String>,
    pub is_required: bool,
    pub struct_name: String,
}

impl ElementBase {
    pub fn new(
        path: &Path,
        description: Option<String>,
        is_required: bool,
        struct_name: &str,
    ) -> Self {
        let title = description
            .as_ref()
            .and_then(|d| d.split('\n').next())
            .map(String::from)
            .unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string()
            });

        Self {
            path: path.into(),
            title,
            help: description,
            is_required,
            struct_name: struct_name.to_string(),
        }
    }

    pub fn key(&self) -> String {
        self.path
            .iter()
            .map(|s| format!("{}", s.display()))
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn field_name(&self) -> String {
        self.path
            .iter()
            .next_back()
            .map(|s| format!("{}", s.display()))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub enum ElementType {
    Menu(Menu),
    OneOf(OneOf),
    Item(Item), // Other element types can be added here
}

impl Deref for ElementType {
    type Target = ElementBase;

    fn deref(&self) -> &Self::Target {
        match self {
            ElementType::Menu(menu) => &menu.base,
            ElementType::OneOf(one_of) => &one_of.base,
            ElementType::Item(item) => &item.base,
        }
    }
}

impl DerefMut for ElementType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ElementType::Menu(menu) => &mut menu.base,
            ElementType::OneOf(one_of) => &mut one_of.base,
            ElementType::Item(item) => &mut item.base,
        }
    }
}

impl ElementType {
    pub fn update_from_value(
        &mut self,
        value: &Value,
        struct_name: Option<&str>,
    ) -> Result<(), SchemaError> {
        match self {
            ElementType::Menu(menu) => {
                if let Some(name) = struct_name
                    && menu.struct_name.as_str() != name
                {
                    return Err(SchemaError::TypeMismatch {
                        path: menu.key(),
                        expected: name.to_string(),
                        actual: menu.struct_name.clone(),
                    });
                }
                menu.update_from_value(value)
            }
            ElementType::OneOf(one_of) => one_of.update_from_value(value),
            ElementType::Item(item) => item.update_from_value(value),
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            ElementType::Menu(menu) => menu.is_none(),
            ElementType::OneOf(one_of) => one_of.is_none(),
            ElementType::Item(item) => match &item.item_type {
                super::item::ItemType::String { value, .. } => value.is_none(),
                super::item::ItemType::Number { value, .. } => value.is_none(),
                super::item::ItemType::Integer { value, .. } => value.is_none(),
                super::item::ItemType::Boolean { .. } => false,
                super::item::ItemType::Enum(enum_item) => enum_item.value.is_none(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_base_key() {
        let eb = ElementBase {
            path: PathBuf::from("a").join("b").join("c"),
            ..Default::default()
        };

        assert_eq!(eb.key(), "a.b.c");
        assert_eq!(eb.field_name(), "c");
    }
}
