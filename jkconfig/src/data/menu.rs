use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use log::trace;
use serde_json::Value;

use crate::data::{
    schema::SchemaError,
    types::{ElementBase, ElementType},
};

#[derive(Clone)]
pub struct MenuRoot {
    pub schema_version: String,
    pub title: String,
    pub menu: ElementType,
}

impl MenuRoot {
    pub fn menu(&self) -> &Menu {
        match &self.menu {
            ElementType::Menu(menu) => menu,
            _ => panic!("Root element is not a Menu"),
        }
    }

    pub fn menu_mut(&mut self) -> &mut Menu {
        match &mut self.menu {
            ElementType::Menu(menu) => menu,
            _ => panic!("Root element is not a Menu"),
        }
    }

    pub fn get_by_key(&self, key: &str) -> Option<&ElementType> {
        if key.is_empty() {
            return Some(&self.menu);
        }

        let ks = key.split(".").collect::<Vec<_>>();
        self.menu().get_by_field_path(&ks)
    }

    pub fn get_mut_by_key(&mut self, key: &str) -> Option<&mut ElementType> {
        if key.is_empty() {
            return Some(&mut self.menu);
        }
        let ks = key.split(".").collect::<Vec<_>>();
        self.menu_mut().get_mut_by_field_path(&ks)
    }

    pub fn update_by_value(&mut self, value: &Value) -> Result<(), SchemaError> {
        self.menu.update_from_value(value, None)
    }

    pub fn as_json(&self) -> Value {
        self.menu().as_json()
    }
}

impl Debug for MenuRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuRoot")
            .field("schema_version", &self.schema_version)
            .field("title", &self.title)
            .field("path", &self.menu.path)
            .field("help", &self.menu.help)
            .field("is_required", &self.menu.is_required)
            .field("struct_name", &self.menu.struct_name)
            .field("is_set", &self.menu().is_set)
            .field("children", &self.menu().children)
            .finish()
    }
}

/// Menu => type: object
#[derive(Clone)]
pub struct Menu {
    pub base: ElementBase,
    pub children: HashMap<String, ElementType>,
    pub is_set: bool,
}

impl Menu {
    pub fn as_json(&self) -> Value {
        let mut result = serde_json::Map::new();

        for (child_key, child_element) in &self.children {
            match child_element {
                ElementType::Menu(menu) => {
                    let field_name = menu.field_name();
                    result.insert(field_name, menu.as_json());
                }
                ElementType::Item(item) => {
                    let field_name = item.base.field_name();
                    result.insert(field_name, item.as_json());
                }
                ElementType::OneOf(oneof) => {
                    // For OneOf, the as_json() method already generates the correct structure
                    // with the proper field name, so we should merge its result directly
                    match oneof.as_json() {
                        Value::Object(oneof_result) => {
                            // Merge the OneOf result into our result
                            for (key, value) in oneof_result {
                                result.insert(key, value);
                            }
                        }
                        other => {
                            // For non-object results (like simple strings or null),
                            // use the child_key as the key
                            result.insert(child_key.clone(), other);
                        }
                    }
                }
            }
        }

        Value::Object(result)
    }

    pub fn get_by_field_path(&self, field_path: &[&str]) -> Option<&ElementType> {
        if field_path.is_empty() {
            return None;
        }
        info!("menu get by field path: {:?}", field_path);
        let first_field = field_path[0];

        let child = self.children.get(first_field)?;

        if field_path.len() == 1 {
            return Some(child);
        }

        match child {
            ElementType::Menu(menu) => menu.get_by_field_path(&field_path[1..]),
            ElementType::OneOf(oneof) => oneof.get_by_field_path(&field_path[1..]),
            _ => None,
        }
    }

    pub fn get_mut_by_field_path(&mut self, field_path: &[&str]) -> Option<&mut ElementType> {
        if field_path.is_empty() {
            return None;
        }

        let first_field = field_path[0];

        let child = self.children.get_mut(first_field)?;

        if field_path.len() == 1 {
            return Some(child);
        }

        match child {
            ElementType::Menu(menu) => menu.get_mut_by_field_path(&field_path[1..]),
            ElementType::OneOf(oneof) => oneof.get_mut_by_field_path(&field_path[1..]),
            _ => None,
        }
    }

    pub fn update_from_value(&mut self, value: &Value) -> Result<(), SchemaError> {
        let value = value.as_object().ok_or(SchemaError::TypeMismatch {
            path: self.key(),
            expected: "object".to_string(),
            actual: serde_json::to_string_pretty(value).unwrap(),
        })?;
        trace!("Updating Menu at {} with value: {:?}", self.key(), value);
        for (key, val) in value {
            if let Some(element) = self.children.get_mut(key) {
                element.update_from_value(val, None)?;
                trace!("Updated child {} of Menu at {}", key, self.key());
            }
            // If key doesn't exist in menu children, skip it as per requirement
        }

        Ok(())
    }

    pub fn is_none(&self) -> bool {
        if self.is_required {
            return false;
        }
        !self.is_set
    }

    pub fn fields(&self) -> Vec<ElementType> {
        self.children
            .values()
            .cloned()
            .collect::<Vec<ElementType>>()
    }
}

impl Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu")
            .field("path", &self.path)
            .field("title", &self.title)
            .field("help", &self.help)
            .field("is_required", &self.is_required)
            .field("is_set", &self.is_set)
            .field("struct_name", &self.struct_name)
            .field("children", &self.children)
            .finish()
    }
}

impl Deref for Menu {
    type Target = ElementBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for Menu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
