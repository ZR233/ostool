use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::data::{
    schema::SchemaError,
    types::{ElementBase, ElementType},
};

use serde_json::Value;

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

    /// Try to determine which variant matches the given value structure
    fn find_matching_variant(&self, value: &Value) -> Result<usize, SchemaError> {
        match value {
            Value::Object(map) => {
                // For OneOf, we need to determine which variant matches the structure
                // Each variant typically has a single key that identifies the variant
                for (idx, variant) in self.variants.iter().enumerate() {
                    match variant {
                        ElementType::Menu(menu) => {
                            // Check if any key in the map matches a child menu's field name
                            for key in map.keys() {
                                // Look through children to find a matching field name
                                for (_child_key, child_element) in &menu.children {
                                    if let ElementType::Menu(child_menu) = child_element {
                                        // Check if the key matches this child menu's field name
                                        if child_menu.field_name() == *key {
                                            return Ok(idx);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            // For non-menu variants, try to update directly
                            if let Ok(()) = variant.clone().update_from_value(value) {
                                return Ok(idx);
                            }
                        }
                    }
                }
                Err(SchemaError::TypeMismatch {
                    path: self.base.key(),
                    expected: format!("one of: {:?}",
                        self.variants.iter().filter_map(|v| {
                            if let ElementType::Menu(menu) = v {
                                Some(menu.children.iter().filter_map(|(_k, e)| {
                                    if let ElementType::Menu(child_menu) = e {
                                        Some(child_menu.field_name())
                                    } else {
                                        None
                                    }
                                }).collect::<Vec<_>>())
                            } else {
                                None
                            }
                        }).collect::<Vec<_>>()),
                    actual: format!("object with keys: {:?}", map.keys().collect::<Vec<_>>()),
                })
            }
            _ => {
                // For non-object values, try each variant
                for (idx, variant) in self.variants.iter().enumerate() {
                    if let Ok(()) = variant.clone().update_from_value(value) {
                        return Ok(idx);
                    }
                }
                Err(SchemaError::TypeMismatch {
                    path: self.base.key(),
                    expected: "matching variant".to_string(),
                    actual: format!("{}", value),
                })
            }
        }
    }

    pub fn update_from_value(&mut self, value: &Value) -> Result<(), SchemaError> {
        // Always re-determine which variant should be selected
        let variant_idx = self.find_matching_variant(value)?;

        // Set the selected variant
        self.selected_index = Some(variant_idx);

        // Update the selected variant with the value
        if let Some(variant) = self.variants.get_mut(variant_idx) {
            match value {
                Value::Object(map) => {
                    // For OneOf with single-property objects, extract the inner value
                    if let ElementType::Menu(menu) = variant {
                        // First pass: find the matching field name
                        let mut matching_field = None;
                        {
                            let menu_clone = menu.clone();
                            for (_child_key, child_element) in &menu_clone.children {
                                if let ElementType::Menu(child_menu) = child_element {
                                    let field_name = child_menu.field_name();
                                    if map.contains_key(&field_name) {
                                        matching_field = Some(field_name);
                                        break;
                                    }
                                }
                            }
                        }

                        // Second pass: update the matching child
                        if let Some(field_name) = matching_field {
                            if let Some(inner_val) = map.get(&field_name) {
                                for (_child_key_mut, child_element_mut) in &mut menu.children {
                                    if let ElementType::Menu(child_menu_mut) = child_element_mut {
                                        if child_menu_mut.field_name() == field_name {
                                            return child_menu_mut.update_from_value(inner_val);
                                        }
                                    }
                                }
                            }
                        }

                        // If no matching child found, update with the whole object
                        menu.update_from_value(value)
                    } else {
                        // For non-menu variants, update directly
                        variant.update_from_value(value)
                    }
                }
                _ => {
                    variant.update_from_value(value)
                }
            }
        } else {
            Err(SchemaError::TypeMismatch {
                path: self.base.key(),
                expected: format!("valid variant index 0-{}", self.variants.len() - 1),
                actual: format!("{}", variant_idx),
            })
        }
    }
}

impl OneOf {
    pub fn field_name(&self) -> String {
        self.base.field_name()
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
