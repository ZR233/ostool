use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
    path::Path,
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
        todo!();
        Ok(())
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
