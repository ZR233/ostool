use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use serde_json::Value;

use crate::data::{
    schema::SchemaError,
    types::{ElementBase, ElementType},
};

#[derive(Clone)]
pub struct MenuRoot {
    pub schema_version: String,
    pub title: String,
    pub menu: Menu,
}

impl MenuRoot {
    pub fn get_by_key(&self, key: &str) -> Option<ElementType> {
        self.menu.get_by_key(key)
    }

    pub fn update_by_value(&mut self, value: &Value) -> Result<(), SchemaError> {
        self.menu.update_from_value(value)
    }

    pub fn as_json(&self) -> Value {
        self.menu.as_json()
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
            .field("children", &self.menu.children)
            .field("is_set", &self.menu.is_set)
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

    pub fn get_by_key(&self, key: &str) -> Option<ElementType> {
        if let Some(v) = self.children.get(key) {
            return Some(v.clone());
        }

        for v in self.children.values() {
            match v {
                ElementType::Menu(menu) => {
                    if let Some(v) = menu.get_by_key(key) {
                        return Some(v);
                    }
                }
                ElementType::OneOf(oneof) => {
                    if let Some(v) = oneof.get_by_key(key) {
                        return Some(v);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn update_from_value(&mut self, value: &Value) -> Result<(), SchemaError> {
        match value {
            Value::Object(map) => {
                // First pass: collect mappings from JSON keys to child keys
                let mut key_mappings = Vec::new();
                {
                    for (child_key, child_element) in &self.children {
                        if let ElementType::Menu(child_menu) = child_element {
                            key_mappings.push((child_menu.field_name(), child_key.clone()));
                        } else if let ElementType::Item(item) = child_element {
                            key_mappings.push((item.base.field_name(), child_key.clone()));
                        } else if let ElementType::OneOf(oneof) = child_element {
                            key_mappings.push((oneof.field_name(), child_key.clone()));
                        }
                    }
                }

                // Second pass: update elements using the mappings
                for (key, val) in map {
                    let mut found_child_key = None;
                    for (field_name, child_key) in &key_mappings {
                        if *field_name == *key {
                            found_child_key = Some(child_key);
                            break;
                        }
                    }

                    if let Some(child_key) = found_child_key
                        && let Some(element) = self.children.get_mut(child_key)
                    {
                        element.update_from_value(val)?;
                    }
                    // If key doesn't exist in menu children, skip it as per requirement
                }
                Ok(())
            }
            _ => Err(SchemaError::TypeMismatch {
                path: self.base.key(),
                expected: "object".to_string(),
                actual: format!("{}", value),
            }),
        }
    }
}

impl Debug for Menu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Menu")
            .field("path", &self.path)
            .field("title", &self.title)
            .field("help", &self.help)
            .field("is_required", &self.is_required)
            .field("children", &self.children)
            .field("is_set", &self.is_set)
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
