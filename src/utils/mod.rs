use serde_json::Value;

pub mod fmt;
pub mod fs;
pub mod mem;
pub mod path;
pub mod tar;

pub fn remove_value_from_json_map(
    path: &str,
    json_map: &mut serde_json::Map<String, Value>,
) -> Vec<Value> {
    match path.split_once('.') {
        Some((element, rest_path)) => match parse_array_path(element) {
            Some((array_element_path, array_index)) => {
                delete_array_path(array_element_path, array_index, Some(rest_path), json_map)
            }
            None => {
                if rest_path.is_empty() {
                    json_map.remove(element).map_or(Vec::new(), |v| vec![v])
                } else {
                    match json_map.get_mut(element) {
                        None => Vec::new(),
                        Some(Value::Object(map)) => remove_value_from_json_map(rest_path, map),
                        Some(_value) => Vec::new(),
                    }
                }
            }
        },
        None => match parse_array_path(path) {
            Some((array_element_path, array_index)) => {
                delete_array_path(array_element_path, array_index, None, json_map)
            }
            None => json_map.remove(path).map_or(Vec::new(), |v| vec![v]),
        },
    }
}

/// Parse array path and index from path
///
/// return Some((path, Some(index))) if path is an array path with index
fn parse_array_path(path: &str) -> Option<(&str, Option<usize>)> {
    // shortcut no array path
    if !path.contains('[') || !path.ends_with(']') {
        return None;
    }
    let mut path = path.split('[');
    let element = path.next();
    let index = path.next();
    match (element, index) {
        (Some(element), None) => Some((element, None)), // no index info
        (Some(element), Some("]")) => Some((element, None)), // full array
        (Some(element), Some(index)) => {
            let trimmed_index = index.trim_matches(']');
            // get numeric index
            match trimmed_index.parse::<usize>() {
                Ok(num_index) => Some((element, Some(num_index))),
                Err(_) => None, // not a well formed path array
            }
        }
        _ => None,
    }
}

fn delete_array_path(
    array_element_path: &str,
    array_index: Option<usize>,
    rest_path: Option<&str>,
    json_map: &mut serde_json::Map<String, Value>,
) -> Vec<Value> {
    match json_map.get_mut(array_element_path) {
        Some(Value::Array(arr)) => match array_index {
            Some(index) if index < arr.len() => {
                if let Some(rest_path) = rest_path {
                    remove_value_from_json_map(rest_path, arr[index].as_object_mut().unwrap())
                } else {
                    vec![arr.remove(index)]
                }
            }
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}
