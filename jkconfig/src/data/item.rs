use crate::data::{schema::SchemaError, types::ElementBase};

use serde_json::Value;

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
    Array(ArrayItem),
}

#[derive(Debug, Clone)]
pub struct ArrayItem {
    /// Array element type (e.g., "string", "integer")
    pub element_type: String,
    /// Array values
    pub values: Vec<String>,
    /// Default values
    pub default: Vec<String>,
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

    pub fn update_from_value(&mut self, value: &Value, path: &str) -> Result<(), SchemaError> {
        match value {
            Value::String(s) => {
                // Try to find the string in variants
                if let Some(idx) = self.variants.iter().position(|v| v == s) {
                    self.value = Some(idx);
                    Ok(())
                } else {
                    Err(SchemaError::TypeMismatch {
                        path: path.to_string(),
                        expected: format!("one of: {:?}", self.variants),
                        actual: s.clone(),
                    })
                }
            }
            Value::Number(n) => {
                if let Some(idx) = n.as_u64() {
                    if (idx as usize) < self.variants.len() {
                        self.value = Some(idx as usize);
                        Ok(())
                    } else {
                        Err(SchemaError::TypeMismatch {
                            path: path.to_string(),
                            expected: format!("index 0-{}", self.variants.len() - 1),
                            actual: format!("{}", idx),
                        })
                    }
                } else {
                    Err(SchemaError::TypeMismatch {
                        path: path.to_string(),
                        expected: "non-negative integer".to_string(),
                        actual: format!("{}", n),
                    })
                }
            }
            _ => Err(SchemaError::TypeMismatch {
                path: path.to_string(),
                expected: "string or number".to_string(),
                actual: format!("{}", value),
            }),
        }
    }
}

impl ItemType {
    pub fn update_from_value(&mut self, value: &Value, path: &str) -> Result<(), SchemaError> {
        match self {
            ItemType::String {
                value: current_value,
                ..
            } => match value {
                Value::String(s) => {
                    *current_value = Some(s.clone());
                    Ok(())
                }
                _ => Err(SchemaError::TypeMismatch {
                    path: path.to_string(),
                    expected: "string".to_string(),
                    actual: format!("{}", value),
                }),
            },
            ItemType::Number {
                value: current_value,
                ..
            } => match value {
                Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        *current_value = Some(f);
                        Ok(())
                    } else {
                        Err(SchemaError::TypeMismatch {
                            path: path.to_string(),
                            expected: "number".to_string(),
                            actual: format!("{}", n),
                        })
                    }
                }
                _ => Err(SchemaError::TypeMismatch {
                    path: path.to_string(),
                    expected: "number".to_string(),
                    actual: format!("{}", value),
                }),
            },
            ItemType::Integer {
                value: current_value,
                ..
            } => match value {
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        *current_value = Some(i);
                        Ok(())
                    } else {
                        Err(SchemaError::TypeMismatch {
                            path: path.to_string(),
                            expected: "integer".to_string(),
                            actual: format!("{}", n),
                        })
                    }
                }
                _ => Err(SchemaError::TypeMismatch {
                    path: path.to_string(),
                    expected: "integer".to_string(),
                    actual: format!("{}", value),
                }),
            },
            ItemType::Boolean {
                value: current_value,
                ..
            } => match value {
                Value::Bool(b) => {
                    *current_value = *b;
                    Ok(())
                }
                _ => Err(SchemaError::TypeMismatch {
                    path: path.to_string(),
                    expected: "boolean".to_string(),
                    actual: format!("{}", value),
                }),
            },
            ItemType::Enum(enum_item) => enum_item.update_from_value(value, path),
            ItemType::Array(array_item) => match value {
                Value::Array(arr) => {
                    let mut values = Vec::new();
                    for item in arr {
                        match item {
                            Value::String(s) => values.push(s.clone()),
                            Value::Number(n) => values.push(n.to_string()),
                            Value::Bool(b) => values.push(b.to_string()),
                            _ => {
                                return Err(SchemaError::TypeMismatch {
                                    path: path.to_string(),
                                    expected: "string, number, or boolean".to_string(),
                                    actual: format!("{}", item),
                                });
                            }
                        }
                    }
                    array_item.values = values;
                    Ok(())
                }
                _ => Err(SchemaError::TypeMismatch {
                    path: path.to_string(),
                    expected: "array".to_string(),
                    actual: format!("{}", value),
                }),
            },
        }
    }
}

impl Item {
    pub fn as_json(&self) -> Value {
        match &self.item_type {
            ItemType::String { value, .. } => match value {
                Some(v) => Value::String(v.clone()),
                None => Value::Null,
            },
            ItemType::Number { value, .. } => match value {
                Some(v) => Value::Number(
                    serde_json::Number::from_f64(*v).unwrap_or(serde_json::Number::from(0)),
                ),
                None => Value::Null,
            },
            ItemType::Integer { value, .. } => match value {
                Some(v) => Value::Number(serde_json::Number::from(*v)),
                None => Value::Null,
            },
            ItemType::Boolean { value, .. } => Value::Bool(*value),
            ItemType::Enum(enum_item) => match enum_item.value_str() {
                Some(v) => Value::String(v.to_string()),
                None => Value::Null,
            },
            ItemType::Array(array_item) => {
                let arr: Vec<Value> = array_item
                    .values
                    .iter()
                    .map(|s| {
                        // Try to parse as number first
                        if let Ok(i) = s.parse::<i64>() {
                            Value::Number(serde_json::Number::from(i))
                        } else if let Ok(f) = s.parse::<f64>() {
                            Value::Number(
                                serde_json::Number::from_f64(f)
                                    .unwrap_or(serde_json::Number::from(0)),
                            )
                        } else if let Ok(b) = s.parse::<bool>() {
                            Value::Bool(b)
                        } else {
                            Value::String(s.clone())
                        }
                    })
                    .collect();
                Value::Array(arr)
            }
        }
    }

    pub fn update_from_value(&mut self, value: &Value) -> Result<(), SchemaError> {
        let path = self.base.key();
        self.item_type.update_from_value(value, &path)
    }
}
