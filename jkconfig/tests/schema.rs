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
