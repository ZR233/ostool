use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::data::types::{ElementBase, ElementType};

#[derive(Clone)]
pub struct MenuRoot {
    pub schema_version: String,
    pub title: String,
    pub menu: Menu,
}

impl MenuRoot {}

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
