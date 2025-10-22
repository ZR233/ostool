use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::data::types::{ElementBase, ElementType};

#[derive(Clone)]
pub struct OneOf {
    pub base: ElementBase,
    pub variants: Vec<ElementType>,
    pub selected_index: Option<usize>,
    pub default_index: Option<usize>,
}

impl OneOf {
    pub fn selected(&self) -> Option<&ElementType> {
        self.selected_index.and_then(|idx| self.variants.get(idx))
    }

    pub fn get_by_key(&self, key: &str) -> Option<ElementType> {
        if let Some(v) = self.selected() {
            if v.key() == key {
                return Some(v.clone());
            }

            match v {
                ElementType::Menu(menu) => {
                    if let Some(v) = menu.get_by_key(key) {
                        return Some(v);
                    }
                }
                ElementType::OneOf(val) => {
                    if let Some(v) = val.get_by_key(key) {
                        return Some(v);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

impl Deref for OneOf {
    type Target = ElementBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for OneOf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Debug for OneOf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OneOf")
            .field("path", &self.path)
            .field("title", &self.title)
            .field("help", &self.help)
            .field("is_required", &self.is_required)
            .field("variants", &self.variants)
            .field("selected_index", &self.selected_index)
            .field("default_index", &self.default_index)
            .finish()
    }
}
