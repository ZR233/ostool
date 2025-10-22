use std::{
    collections::HashSet,
    ops::Deref,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::data::{
    menu::{Menu, MenuRoot},
    oneof::OneOf,
    types::{ElementBase, ElementType},
};

#[derive(thiserror::Error, Debug)]
pub enum SchemaError {
    #[error("Unsupported schema")]
    UnsupportedSchema,
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Schema conversion error at {path:?}: {reason}")]
    SchemaConversionError { path: PathBuf, reason: String },
}

#[derive(Debug, Clone)]
struct WalkContext {
    path: PathBuf,
    value: Value,
    defs: Option<Value>,
}

impl Deref for WalkContext {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl WalkContext {
    fn new(value: Value) -> Self {
        Self {
            path: PathBuf::new(),
            value,
            defs: None,
        }
    }

    fn required_field(&self, field_name: &str) -> Result<&Value, SchemaError> {
        self.get(field_name)
            .ok_or(SchemaError::SchemaConversionError {
                path: self.path.clone(),
                reason: format!("Missing required field '{}'", field_name),
            })
    }

    fn required_field_as_string(&self, field_name: &str) -> Result<String, SchemaError> {
        let value = self.required_field(field_name)?;
        Ok(value
            .as_str()
            .map(String::from)
            .ok_or(SchemaError::SchemaConversionError {
                path: self.path.clone(),
                reason: "$schema is not string".into(),
            })?
            .to_string())
    }

    fn get_str(&self, field_name: &str) -> Result<Option<&str>, SchemaError> {
        self.get(field_name)
            .map(|v| {
                v.as_str().ok_or(SchemaError::SchemaConversionError {
                    path: self.path.clone(),
                    reason: format!("Field '{}' is not a string", field_name),
                })
            })
            .transpose()
    }

    fn description(&self) -> Result<Option<String>, SchemaError> {
        let desc = self
            .get("description")
            .map(|d| {
                d.as_str()
                    .map(String::from)
                    .ok_or(SchemaError::SchemaConversionError {
                        path: self.path.clone(),
                        reason: "description is not a string".into(),
                    })
            })
            .transpose()?;
        Ok(desc)
    }

    fn as_menu(&self, is_required: bool) -> Result<Option<Menu>, SchemaError> {
        if let Some(ty) = self.get("type")
            && let Some(ty_str) = ty.as_str()
            && ty_str == "object"
        {
            let menu = Menu::from_schema(self, is_required)?;
            return Ok(Some(menu));
        }
        Ok(None)
    }

    fn as_ref(&self, is_required: bool) -> Result<Option<ElementType>, SchemaError> {
        if let Some(ref_value) = self.get("$ref")
            && let Some(ref_str) = ref_value.as_str()
        {
            let def_name = ref_str.trim_start_matches("#/$defs/");
            if let Some(defs) = &self.defs
                && let Some(def_value) = defs.get(def_name)
            {
                let mut walk = self.clone();
                walk.value = def_value.clone();
                return walk.as_element_type(is_required);
            }
            // Handle $ref logic here
            // For example, resolve the reference and create the appropriate ElementType
        }
        Ok(None)
    }

    fn as_oneof(&self, is_required: bool) -> Result<Option<OneOf>, SchemaError> {
        if let Some(one_of) = self.get("oneOf")
            && let Some(variants) = one_of.as_array()
        {
            let mut variant_elements = Vec::new();
            for variant in variants {
                // Process each variant
                let mut walk = self.clone();
                walk.value = variant.clone();
                if let Some(element_type) = walk.as_element_type(false)? {
                    variant_elements.push(element_type);
                }
            }

            let one_of = OneOf {
                base: ElementBase::new(&self.path, self.description()?, is_required),
                variants: variant_elements,
                selected_index: None,
                default_index: None,
            };

            return Ok(Some(one_of));
        }

        Ok(None)
    }

    fn as_element_type(&self, is_required: bool) -> Result<Option<ElementType>, SchemaError> {
        if let Some(menu) = self.as_menu(is_required)? {
            return Ok(Some(ElementType::Menu(menu)));
        }

        if let Some(val) = self.as_ref(is_required)? {
            return Ok(Some(val));
        }

        if let Some(one_of) = self.as_oneof(is_required)? {
            return Ok(Some(ElementType::OneOf(one_of)));
        }
        // Handle other ElementType variants here
        Ok(None)
    }
}

impl TryFrom<&Value> for MenuRoot {
    type Error = SchemaError;

    fn try_from(schema: &Value) -> Result<Self, Self::Error> {
        let mut walk = WalkContext::new(schema.clone());
        let schema_version = walk.required_field_as_string("$schema")?;
        let title = walk.required_field_as_string("title")?;

        walk.defs = walk.get("$defs").cloned();

        let menu = Menu::from_schema(&walk, true)?;

        Ok(MenuRoot {
            schema_version,
            title,
            menu,
        })
    }
}

impl Menu {
    fn from_schema(walk: &WalkContext, is_required: bool) -> Result<Self, SchemaError> {
        let description = walk.description()?;

        let mut menu = Menu {
            base: ElementBase::new(&walk.path, description, is_required),
            children: Default::default(),
            is_set: is_required,
        };

        let mut required_fields = HashSet::new();

        if let Some(req) = walk.get("required")
            && let Some(req_array) = req.as_array()
        {
            for item in req_array {
                if let Some(field_name) = item.as_str() {
                    required_fields.insert(field_name.to_string());
                }
            }
        }

        if let Some(properties) = walk.get("properties")
            && let Some(props_map) = properties.as_object()
        {
            for (key, value) in props_map {
                let child_path = walk.path.join(key);
                let is_required = required_fields.contains(key);
                let walk = WalkContext {
                    path: child_path,
                    value: value.clone(),
                    defs: walk.defs.clone(),
                };

                menu.handle_children(&walk, is_required)?;
            }
        }

        // Placeholder implementation
        Ok(menu)
    }

    fn handle_children(
        &mut self,
        walk: &WalkContext,
        is_required: bool,
    ) -> Result<(), SchemaError> {
        if let Some(val) = walk.as_element_type(is_required)? {
            self.children.insert(val.key(), val);
        }
        Ok(())
    }
}
