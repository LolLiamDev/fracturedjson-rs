use crate::error::FracturedJsonError;
use crate::model::{JsonItem, JsonItemType};

pub fn convert_value_to_dom(
    element: &serde_json::Value,
    prop_name: Option<&str>,
    recursion_limit: usize,
) -> Result<Option<JsonItem>, FracturedJsonError> {
    if recursion_limit == 0 {
        return Err(FracturedJsonError::simple(
            "Depth limit exceeded - possible circular reference",
        ));
    }

    let mut item = JsonItem::default();
    if let Some(name) = prop_name {
        item.name = serde_json::to_string(name).unwrap_or_else(|_| format!("\"{}\"", name));
    }

    match element {
        serde_json::Value::Null => {
            item.item_type = JsonItemType::Null;
            item.value = "null".to_string();
        }
        serde_json::Value::Bool(val) => {
            item.item_type = if *val { JsonItemType::True } else { JsonItemType::False };
            item.value = if *val { "true".to_string() } else { "false".to_string() };
        }
        serde_json::Value::Number(num) => {
            item.item_type = JsonItemType::Number;
            item.value = num.to_string();
        }
        serde_json::Value::String(val) => {
            item.item_type = JsonItemType::String;
            item.value = serde_json::to_string(val).unwrap_or_else(|_| format!("\"{}\"", val));
        }
        serde_json::Value::Array(arr) => {
            item.item_type = JsonItemType::Array;
            let mut children = Vec::with_capacity(arr.len());
            for child in arr {
                let converted = convert_value_to_dom(child, None, recursion_limit - 1)?;
                if let Some(child_item) = converted {
                    children.push(child_item);
                } else {
                    let null_item = convert_value_to_dom(&serde_json::Value::Null, None, recursion_limit - 1)?;
                    if let Some(null_item) = null_item {
                        children.push(null_item);
                    }
                }
            }
            item.children = children;
        }
        serde_json::Value::Object(map) => {
            item.item_type = JsonItemType::Object;
            for (key, value) in map.iter() {
                let child = convert_value_to_dom(value, Some(key), recursion_limit - 1)?;
                if let Some(child_item) = child {
                    item.children.push(child_item);
                }
            }
        }
    }

    if !item.children.is_empty() {
        let highest_child_complexity = item
            .children
            .iter()
            .map(|ch| ch.complexity)
            .max()
            .unwrap_or(0);
        item.complexity = highest_child_complexity + 1;
    }

    Ok(Some(item))
}
