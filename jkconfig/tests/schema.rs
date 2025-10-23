use jkconfig::data::menu::MenuRoot;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

// Use Animal structures from other test file
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Cat {
    pub a: usize,
    pub b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dog {
    pub c: Option<f32>,
    pub d: bool,
    pub l: Legs,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Legs {
    Four,
    Two,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// 动物类型
/// Cat 或 Dog 的枚举
pub enum AnimalEnum {
    Cat(Cat),
    Dog(Dog),
    Rabbit,
    Duck { h: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct AnimalObject {
    animal: AnimalEnum,
}

#[test]
fn test_object() {
    let schema = schema_for!(AnimalObject);
    println!(
        "Generated JSON Schema: \n{}",
        serde_json::to_string_pretty(&schema).unwrap()
    );
    let menu = MenuRoot::try_from(schema.as_value()).unwrap();

    println!("Generated MenuRoot: \n{:#?}", menu);

    println!(
        "AnimalEnum element: \n{:#?}",
        menu.get_by_key("animal").unwrap()
    );
}

#[test]
fn test_value() {
    let schema = schema_for!(AnimalObject);

    let origin = AnimalObject {
        animal: AnimalEnum::Dog(Dog {
            c: Some(3.5),
            d: true,
            l: Legs::Four,
        }),
    };

    let value = schema.as_value();

    println!(
        "Generated JSON Schema Value: \n{}",
        serde_json::to_string_pretty(&value).unwrap()
    );

    let mut menu = MenuRoot::try_from(value).unwrap();

    let value = serde_json::to_value(&origin).unwrap();

    menu.update_by_value(&value).unwrap();

    println!("Updated MenuRoot: \n{:#?}", menu);
}

#[test]
fn test_value_normal_case() {
    let schema = schema_for!(AnimalObject);
    let mut menu = MenuRoot::try_from(schema.as_value()).unwrap();

    // Test normal case with correct types
    let dog_value = serde_json::json!({
        "animal": {
            "Dog": {
                "c": 2.7,
                "d": false
            }
        }
    });

    let result = menu.update_by_value(&dog_value);
    assert!(result.is_ok(), "Normal case should succeed: {:?}", result.err());
}

#[test]
fn test_value_type_mismatch() {
    let schema = schema_for!(AnimalObject);
    let mut menu = MenuRoot::try_from(schema.as_value()).unwrap();

    // Test type mismatch: boolean field receives string
    let bad_value = serde_json::json!({
        "animal": {
            "Dog": {
                "c": 2.7,
                "d": "this should be boolean"  // Type mismatch
            }
        }
    });

    let result = menu.update_by_value(&bad_value);
    assert!(result.is_err());
    match &result {
        Err(jkconfig::data::schema::SchemaError::TypeMismatch { path, expected, actual: _ }) => {
            assert!(path.contains("animal.Dog.d"));
            assert_eq!(expected, "boolean");
        }
        _ => panic!("Expected TypeMismatch error but got: {:?}", result),
    }
}

#[test]
fn test_value_skip_unknown_fields() {
    let schema = schema_for!(AnimalObject);
    let mut menu = MenuRoot::try_from(schema.as_value()).unwrap();

    // Test with extra fields that don't exist in schema
    let value_with_extra = serde_json::json!({
        "animal": {
            "Dog": {
                "c": 1.5,
                "d": true,
                "unknown_field": "should be skipped",
                "another_unknown": 42
            }
        },
        "unknown_top_level": "should be skipped"
    });

    let result = menu.update_by_value(&value_with_extra);
    assert!(result.is_ok(), "Should skip unknown fields and succeed");
}

#[test]
fn test_value_empty_object() {
    let schema = schema_for!(AnimalObject);
    let mut menu = MenuRoot::try_from(schema.as_value()).unwrap();

    // Test with empty object (should skip since no matching fields)
    let empty_value = serde_json::json!({});

    let result = menu.update_by_value(&empty_value);
    // This might fail because animal is required, but let's see what happens
    println!("Empty object result: {:?}", result);
}

#[test]
fn test_value_cat_variant() {
    let schema = schema_for!(AnimalObject);
    let mut menu = MenuRoot::try_from(schema.as_value()).unwrap();

    // Test Cat variant
    let cat_value = serde_json::json!({
        "animal": {
            "Cat": {
                "a": 42,
                "b": "meow"
            }
        }
    });

    let result = menu.update_by_value(&cat_value);
    assert!(result.is_ok(), "Cat variant should succeed");
}

#[test]
fn test_value_integer_type_mismatch() {
    let schema = schema_for!(AnimalObject);
    let mut menu = MenuRoot::try_from(schema.as_value()).unwrap();

    // Test integer field receiving float
    let cat_value = serde_json::json!({
        "animal": {
            "Cat": {
                "a": 3.14,  // Should be integer, not float
                "b": "test"
            }
        }
    });

    let result = menu.update_by_value(&cat_value);
    assert!(result.is_err());
    match result.err().unwrap() {
        jkconfig::data::schema::SchemaError::TypeMismatch { path, expected, actual } => {
            assert!(path.contains("animal.Cat.a"));
            assert_eq!(expected, "integer");
        }
        _ => panic!("Expected TypeMismatch error for integer field"),
    }
}

/***
```json
Generated JSON Schema:
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AnimalObject",
  "type": "object",
  "properties": {
    "animal": {
      "$ref": "#/$defs/AnimalEnum"
    }
  },
  "required": [
    "animal"
  ],
  "$defs": {
    "AnimalEnum": {
      "oneOf": [
        {
          "type": "object",
          "properties": {
            "Cat": {
              "$ref": "#/$defs/Cat"
            }
          },
          "additionalProperties": false,
          "required": [
            "Cat"
          ]
        },
        {
          "type": "object",
          "properties": {
            "Dog": {
              "$ref": "#/$defs/Dog"
            }
          },
          "additionalProperties": false,
          "required": [
            "Dog"
          ]
        }
      ]
    },
    "Cat": {
      "type": "object",
      "properties": {
        "a": {
          "type": "integer",
          "format": "uint",
          "minimum": 0
        },
        "b": {
          "type": "string"
        }
      },
      "required": [
        "a",
        "b"
      ]
    },
    "Dog": {
      "type": "object",
      "properties": {
        "c": {
          "type": "number",
          "format": "float"
        },
        "d": {
          "type": "boolean"
        }
      },
      "required": [
        "c",
        "d"
      ]
    }
  }
}
```
***/
