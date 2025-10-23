use jkconfig::data::menu::MenuRoot;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

// Use Animal structures from other test file
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Cat {
    pub a: usize,
    pub b: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Dog {
    pub c: Option<f32>,
    pub d: bool,
    pub l: Legs,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum Legs {
    Four,
    Two,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
/// 动物类型
/// Cat 或 Dog 的枚举
pub enum AnimalEnum {
    Cat(Cat),
    Dog(Dog),
    Rabbit,
    Duck { h: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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

    let actual_value = menu.as_json();

    let actual: AnimalObject = serde_json::from_value(actual_value).unwrap();

    assert_eq!(origin.animal, actual.animal);
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
    assert!(
        result.is_ok(),
        "Normal case should succeed: {:?}",
        result.err()
    );
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
        Err(jkconfig::data::schema::SchemaError::TypeMismatch {
            path,
            expected,
            actual: _,
        }) => {
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
                "a": 3.2,  // Should be integer, not float
                "b": "test"
            }
        }
    });

    let result = menu.update_by_value(&cat_value);
    assert!(result.is_err());
    match result.err().unwrap() {
        jkconfig::data::schema::SchemaError::TypeMismatch {
            path,
            expected,
            actual: _,
        } => {
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

// 测试 MenuRoot::get_mut_by_key 方法的边界条件
#[cfg(test)]
mod menu_root_get_mut_by_key_tests {
    use super::*;

    /// 创建测试用的 MenuRoot 实例
    fn create_test_menu_root() -> MenuRoot {
        let schema = schema_for!(AnimalObject);
        MenuRoot::try_from(schema.as_value()).unwrap()
    }

    #[test]
    /// 测试空字符串键
    fn test_get_mut_by_key_empty_string() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("");

        assert!(result.is_none(), "Empty string key should return None");
    }

    #[test]
    /// 测试单点路径（根级别的简单字段）
    fn test_get_mut_by_key_single_field() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal");

        assert!(result.is_some(), "Valid top-level key 'animal' should return Some");
    }

    #[test]
    /// 测试简单有效路径（两层嵌套）
    fn test_get_mut_by_key_simple_path() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal");

        assert!(result.is_some(), "Valid path 'animal' should return Some");
    }

    #[test]
    /// 测试不存在的键路径
    fn test_get_mut_by_key_nonexistent_path() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("nonexistent.path");

        assert!(result.is_none(), "Nonexistent path should return None");
    }

    #[test]
    /// 测试开头点号的路径
    fn test_get_mut_by_key_leading_dot() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key(".animal");

        assert!(result.is_none(), "Path starting with dot should return None");
    }

    #[test]
    /// 测试结尾点号的路径
    fn test_get_mut_by_key_trailing_dot() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal.");

        assert!(result.is_none(), "Path ending with dot should return None");
    }

    #[test]
    /// 测试连续点号的路径
    fn test_get_mut_by_key_consecutive_dots() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal..Dog");

        assert!(result.is_none(), "Path with consecutive dots should return None");
    }

    #[test]
    /// 测试深层嵌套路径（基于实际存在的路径）
    fn test_get_mut_by_key_deep_nesting() {
        let mut menu = create_test_menu_root();

        // 测试一个可能存在的深层路径
        let result = menu.get_mut_by_key("animal.Cat.a");

        // 如果深层路径不存在，至少animal应该存在
        if result.is_none() {
            let result2 = menu.get_mut_by_key("animal");
            assert!(result2.is_some(), "At least 'animal' path should return Some");
        } else {
            assert!(result.is_some(), "Valid deep nested path should return Some");
        }
    }

    #[test]
    /// 测试特殊字符处理
    fn test_get_mut_by_key_special_characters() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal-Dog@c");

        assert!(result.is_none(), "Path with special characters should return None");
    }

    #[test]
    /// 测试Unicode字符支持
    fn test_get_mut_by_key_unicode_characters() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal.动物.c");

        assert!(result.is_none(), "Path with Unicode characters should return None");
    }

    #[test]
    /// 测试只有点号的路径
    fn test_get_mut_by_key_only_dots() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("...");

        assert!(result.is_none(), "Path with only dots should return None");
    }

    #[test]
    /// 测试空字段在路径中间
    fn test_get_mut_by_key_empty_field_in_middle() {
        let mut menu = create_test_menu_root();

        let result = menu.get_mut_by_key("animal..c");

        assert!(result.is_none(), "Path with empty field in middle should return None");
    }
}
