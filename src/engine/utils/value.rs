use serde_json::Value;

/// Avoids allocating Vec with a single element
#[derive(Debug)]
pub enum MultiValue<T> {
    One(Option<T>),
    Many(Vec<T>),
}

impl<T> Default for MultiValue<T> {
    fn default() -> Self {
        Self::One(None)
    }
}

impl<T> MultiValue<T> {
    pub(crate) fn one(value: T) -> Self {
        Self::One(Some(value))
    }

    fn option(value: Option<T>) -> Self {
        Self::One(value)
    }

    fn push(&mut self, value: T) {
        match self {
            Self::One(opt) => match opt.take() {
                Some(v) => {
                    *self = Self::Many(vec![v, value]);
                }
                None => {
                    *self = Self::One(Some(value));
                }
            },
            Self::Many(vec) => {
                vec.push(value);
            }
        }
    }

    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.push(value);
        }
    }

    pub fn values(self) -> Vec<T> {
        match self {
            Self::One(opt) => opt.into_iter().collect(),
            Self::Many(vec) => vec,
        }
    }

    #[cfg(test)]
    pub(crate) fn as_ref(&self) -> MultiValue<&T> {
        match self {
            Self::One(opt) => MultiValue::option(opt.as_ref()),
            Self::Many(vec) => MultiValue::Many(vec.iter().collect()),
        }
    }
}

impl MultiValue<&Value> {
    pub(crate) fn check_is_empty(&self) -> bool {
        match self {
            Self::Many(vec) => vec.iter().all(|x| match x {
                Value::Array(vec) => vec.is_empty(),
                Value::Null => true,
                _ => false,
            }),
            Self::One(val) => match val {
                None => true,
                Some(Value::Array(vec)) => vec.is_empty(),
                Some(Value::Null) => true,
                _ => false,
            },
        }
    }

    pub(crate) fn check_is_null(&self) -> bool {
        match self {
            MultiValue::One(val) => {
                if let Some(val) = val {
                    return val.is_null();
                }
                false
            }
            // { "a": [ { "b": null }, { "b": 1 } ] } => true
            // { "a": [ { "b": 1 }, { "b": null } ] } => true
            // { "a": [ { "b": 1 }, { "b": 2 } ] } => false
            MultiValue::Many(vals) => vals.iter().any(|val| val.is_null()),
        }
    }
}

impl<T> IntoIterator for MultiValue<T> {
    type Item = T;
    // propagate to Vec internal iterator
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::One(None) => vec![].into_iter(),
            Self::One(Some(a)) => vec![a].into_iter(),
            Self::Many(vec) => vec.into_iter(),
        }
    }
}

pub fn rev_range(a: usize, b: usize) -> impl Iterator<Item = usize> {
    (b + 1..=a).rev()
}

/// Parse array path and index from path
///
/// return Some((path, Some(index))) if path is an array path with index
fn parse_array_path(path: &str) -> Option<(&str, Option<u32>)> {
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
            match trimmed_index.parse::<u32>() {
                Ok(num_index) => Some((element, Some(num_index))),
                Err(_) => None, // not a well-formed path array
            }
        }
        _ => None,
    }
}

/// Focus on array values references according to array path
///
/// Expects to be called with a path that is a path to an Array
fn focus_array_path<'a>(
    array_path: &str,
    array_index: Option<u32>,
    rest_of_path: Option<&str>,
    json_map: &'a serde_json::Map<String, Value>,
) -> Option<MultiValue<&'a Value>> {
    match json_map.get(array_path) {
        Some(Value::Array(array)) => {
            let mut values: MultiValue<_> = MultiValue::default();
            for (i, value) in array.iter().enumerate() {
                if let Some(array_index) = array_index {
                    if i == array_index as usize {
                        match rest_of_path {
                            Some(rest_path) => {
                                // expect an Object if there is a rest path
                                if let Value::Object(map) = value {
                                    values.extend(get_value_from_json_map(rest_path, map))
                                }
                            }
                            None => values.push(value),
                        }
                    }
                } else {
                    match rest_of_path {
                        Some(rest_path) => {
                            // expect an Object if there is a rest path
                            if let Value::Object(map) = value {
                                values.extend(get_value_from_json_map(rest_path, map))
                            }
                        }
                        None => values.push(value),
                    }
                }
            }
            Some(values)
        }
        _ => None,
    }
}

pub fn get_value_from_json_map_opt<'a>(
    path: &str,
    json_map: &'a serde_json::Map<String, Value>,
) -> Option<MultiValue<&'a Value>> {
    // check if leaf path element
    match path.split_once('.') {
        Some((element, rest_path)) => {
            // check if targeting array
            match parse_array_path(element) {
                Some((array_element_path, array_index)) => {
                    focus_array_path(array_element_path, array_index, Some(rest_path), json_map)
                }
                None => {
                    // no array notation
                    match json_map.get(element) {
                        Some(Value::Object(map)) => get_value_from_json_map_opt(rest_path, map),
                        Some(value) => rest_path.is_empty().then_some(MultiValue::one(value)),
                        None => None,
                    }
                }
            }
        }
        None => match parse_array_path(path) {
            Some((array_element_path, array_index)) => {
                focus_array_path(array_element_path, array_index, None, json_map)
            }
            None => json_map.get(path).map(MultiValue::one),
        },
    }
}

/// Focus on value references according to path
/// Flatten intermediate arrays but keep leaf array values on demand.
/// E.g
/// {
///   "arr": [
///       { "a": [1, 2, 3] },
///       { "a": 4 },
///       { "b": 5 }
///   ]
/// }
///
/// path: "arr[].a"   => Vec![Value::Array[ 1, 2, 3], 4]
/// path: "arr[].a[]" => Vec![ 1, 2, 3, 4]
///
/// performance: the function could be improved by using the Entry API instead of BTreeMap.get
pub fn get_value_from_json_map<'a>(
    path: &str,
    json_map: &'a serde_json::Map<String, Value>,
) -> MultiValue<&'a Value> {
    get_value_from_json_map_opt(path, json_map).unwrap_or_default()
}

/// Delete array values according to array path
///
/// Expects to be called with a path that is a path to an Array
fn delete_array_path(
    array_path: &str,
    array_index: Option<u32>,
    rest_of_path: Option<&str>,
    json_map: &mut serde_json::Map<String, Value>,
) -> MultiValue<Value> {
    if let Some(Value::Array(array)) = json_map.get_mut(array_path) {
        match rest_of_path {
            None => {
                // end of path - delete and collect
                if let Some(array_index) = array_index {
                    if array.len() > array_index as usize {
                        return MultiValue::one(array.remove(array_index as usize));
                    }
                } else {
                    return MultiValue::one(Value::Array(std::mem::take(array)));
                }
            }
            Some(rest_path) => {
                // dig deeper
                let mut values = MultiValue::default();
                for (i, value) in array.iter_mut().enumerate() {
                    if let Value::Object(map) = value {
                        if let Some(array_index) = array_index {
                            if i == array_index as usize {
                                values.extend(remove_value_from_json_map(rest_path, map));
                            }
                        } else {
                            values.extend(remove_value_from_json_map(rest_path, map));
                        }
                    }
                }
                return values;
            }
        }
    }
    // no array found
    MultiValue::default()
}

/// Remove value at a given JSON path from JSON map
///
/// performance: the function could be improved by using the Entry API instead of BTreeMap.get_mut
pub fn remove_value_from_json_map(
    path: &str,
    json_map: &mut serde_json::Map<String, Value>,
) -> MultiValue<Value> {
    // check if leaf path element
    match path.split_once('.') {
        Some((element, rest_path)) => {
            // check if targeting array
            match parse_array_path(element) {
                Some((array_element_path, array_index)) => {
                    delete_array_path(array_element_path, array_index, Some(rest_path), json_map)
                }
                None => {
                    // no array notation
                    if rest_path.is_empty() {
                        MultiValue::option(json_map.remove(element))
                    } else {
                        match json_map.get_mut(element) {
                            None => MultiValue::default(),
                            Some(Value::Object(map)) => remove_value_from_json_map(rest_path, map),
                            Some(_value) => MultiValue::default(),
                        }
                    }
                }
            }
        }
        None => match parse_array_path(path) {
            Some((array_element_path, array_index)) => {
                delete_array_path(array_element_path, array_index, None, json_map)
            }
            None => MultiValue::option(json_map.remove(path)),
        },
    }
}

/// Check if a path is included in a list of patterns
///
/// Basically, it checks if either the pattern or path is a prefix of the other.
/// Examples:
/// ```
/// assert!(segment::common::utils::check_include_pattern("a.b.c", "a.b.c"));
/// assert!(segment::common::utils::check_include_pattern("a.b.c", "a.b"));
/// assert!(!segment::common::utils::check_include_pattern("a.b.c", "a.b.d"));
/// assert!(segment::common::utils::check_include_pattern("a.b.c", "a"));
/// assert!(segment::common::utils::check_include_pattern("a", "a.d"));
/// ```
pub fn check_include_pattern(pattern: &str, path: &str) -> bool {
    pattern
        .split(['.', '['])
        .zip(path.split(['.', '[']))
        .all(|(p, v)| p == v)
}

/// Check if a path should be excluded by a pattern
///
/// Basically, it checks if pattern is a prefix of path, but not the other way around.
///
/// ```
/// assert!(segment::common::utils::check_exclude_pattern("a.b.c", "a.b.c"));
/// assert!(!segment::common::utils::check_exclude_pattern("a.b.c", "a.b"));
/// assert!(!segment::common::utils::check_exclude_pattern("a.b.c", "a.b.d"));
/// assert!(!segment::common::utils::check_exclude_pattern("a.b.c", "a"));
/// assert!(segment::common::utils::check_exclude_pattern("a", "a.d"));
/// ```

pub fn check_exclude_pattern(pattern: &str, path: &str) -> bool {
    if pattern.len() > path.len() {
        return false;
    }
    pattern
        .split(['.', '['])
        .zip(path.split(['.', '[']))
        .all(|(p, v)| p == v)
}

fn _filter_json_values<'a>(
    mut path: String,
    value: &'a Value,
    filter: &dyn Fn(&str, &Value) -> bool,
) -> (String, Value) {
    let value = match &value {
        Value::Null => value.clone(),
        Value::Bool(_) => value.clone(),
        Value::Number(_) => value.clone(),
        Value::String(_) => value.clone(),
        Value::Array(array) => {
            let mut new_array = Vec::new();
            path.push_str("[]");
            for value in array.iter() {
                if filter(&path, value) {
                    let (path_, value) = _filter_json_values(path, value, filter);
                    path = path_;
                    new_array.push(value);
                }
            }
            path.truncate(path.len() - 2);
            Value::Array(new_array)
        }
        Value::Object(object) => {
            let mut new_object = serde_json::Map::new();
            for (key, value) in object.iter() {
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(key);
                if filter(&path, value) {
                    let (path_, value) = _filter_json_values(path, value, filter);
                    path = path_;
                    new_object.insert(key.clone(), value);
                }
                path.truncate(path.len() - key.len());
                if !path.is_empty() {
                    path.pop();
                }
            }
            Value::Object(new_object)
        }
    };
    (path, value)
}

/// Filter json map based on external filter function
///
/// Filter function takes path and value as input and returns true if the value should be kept
pub fn filter_json_values(
    json_map: &serde_json::Map<String, Value>,
    filter: impl Fn(&str, &Value) -> bool,
) -> serde_json::Map<String, Value> {
    let path = "".to_string();
    let (_, res) = _filter_json_values(path, &Value::Object(json_map.clone()), &filter);

    if let Value::Object(map) = res {
        map
    } else {
        // This should never happen, because _filter_json_values always returns same
        // type as input
        unreachable!("Unexpected value type")
    }
}

/// Light abstraction over a JSON path to avoid concatenating strings
#[derive(Debug, Clone)]
pub struct JsonPathPayload {
    pub path: String,
}

impl JsonPathPayload {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn extend(&self, segment: &str) -> Self {
        let full_path = format!("{}.{}", self.path, segment);
        JsonPathPayload::new(full_path)
    }

    pub fn extend_or_new(base: Option<&Self>, segment: &str) -> Self {
        match base {
            Some(path) => path.extend(segment),
            None => JsonPathPayload::new(segment.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_nested_value_from_json_map() {
        let map = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"
            {
                "a": {
                    "b": {
                        "c": 1
                    }
                },
                "d": 2
            }
            "#,
        )
        .unwrap();

        assert_eq!(
            get_value_from_json_map("a.b", &map).values(),
            vec![&Value::Object(serde_json::Map::from_iter(vec![(
                "c".to_string(),
                Value::Number(1.into())
            )]))]
        );

        // going deeper
        assert_eq!(
            get_value_from_json_map("a.b.c", &map).values(),
            vec![&Value::Number(1.into())]
        );

        // missing path
        assert!(get_value_from_json_map("a.b.c.d", &map).check_is_empty());
    }

    #[test]
    fn test_is_empty() {
        let map = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"
                {
                   "a": [
                     { "b": 1 },
                     { "b": 2 },
                     { "b": null },
                     { "d": [] },
                     { "d": [] },
                     { "f": null }
                   ]
                }
            "#,
        )
        .unwrap();
        let multivalue = get_value_from_json_map("a[].b", &map);
        let is_empty = multivalue.check_is_empty();

        assert!(!is_empty, "a[].b is not empty");

        let multivalue = get_value_from_json_map("a[].c", &map);
        let is_empty = multivalue.check_is_empty();

        assert!(is_empty, "a[].c is empty");

        let multivalue = get_value_from_json_map("a[].d", &map);
        let is_empty = multivalue.check_is_empty();
        assert!(is_empty, "a[].d is empty");

        let multivalue = get_value_from_json_map("a[].f", &map);
        let is_empty = multivalue.check_is_empty();
        assert!(is_empty, "a[].f is empty");
    }

    #[test]
    fn test_get_nested_array_value_from_json_map() {
        let map = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"
            {
                "a": {
                    "b": [
                        { "c": 1 },
                        { "c": 2 },
                        { "d": { "e": 3 } }
                    ]
                },
                "f": 3,
                "g": ["g0", "g1", "g2"]
            }
            "#,
        )
        .unwrap();

        // get JSON array
        assert_eq!(
            get_value_from_json_map("a.b", &map).values(),
            vec![&Value::Array(vec![
                Value::Object(serde_json::Map::from_iter(vec![(
                    "c".to_string(),
                    Value::Number(1.into())
                )])),
                Value::Object(serde_json::Map::from_iter(vec![(
                    "c".to_string(),
                    Value::Number(2.into())
                )])),
                Value::Object(serde_json::Map::from_iter(vec![(
                    "d".to_string(),
                    Value::Object(serde_json::Map::from_iter(vec![(
                        "e".to_string(),
                        Value::Number(3.into())
                    )]))
                )])),
            ])]
        );

        // a.b[] extract all elements from array
        assert_eq!(
            get_value_from_json_map("a.b[]", &map).values(),
            vec![
                &Value::Object(serde_json::Map::from_iter(vec![(
                    "c".to_string(),
                    Value::Number(1.into())
                )])),
                &Value::Object(serde_json::Map::from_iter(vec![(
                    "c".to_string(),
                    Value::Number(2.into())
                )])),
                &Value::Object(serde_json::Map::from_iter(vec![(
                    "d".to_string(),
                    Value::Object(serde_json::Map::from_iter(vec![(
                        "e".to_string(),
                        Value::Number(3.into())
                    )]))
                )])),
            ]
        );

        // project scalar field through array
        assert_eq!(
            get_value_from_json_map("a.b[].c", &map).values(),
            vec![&Value::Number(1.into()), &Value::Number(2.into())]
        );

        // project object field through array
        assert_eq!(
            get_value_from_json_map("a.b[].d", &map).values(),
            vec![&Value::Object(serde_json::Map::from_iter(vec![(
                "e".to_string(),
                Value::Number(3.into())
            )]))]
        );

        // select scalar element from array
        assert_eq!(
            get_value_from_json_map("a.b[0]", &map).values(),
            vec![&Value::Object(serde_json::Map::from_iter(vec![(
                "c".to_string(),
                Value::Number(1.into())
            )]))]
        );

        // select scalar object from array different index
        assert_eq!(
            get_value_from_json_map("a.b[1]", &map).values(),
            vec![&Value::Object(serde_json::Map::from_iter(vec![(
                "c".to_string(),
                Value::Number(2.into())
            )]))]
        );

        // select field element from array different index
        assert_eq!(
            get_value_from_json_map("a.b[1].c", &map).values(),
            vec![&Value::Number(2.into())]
        );

        // select scalar element from array different index
        assert_eq!(
            get_value_from_json_map("g[2]", &map).values(),
            vec![&Value::String("g2".to_string())]
        );

        // select object element from array
        assert_eq!(
            get_value_from_json_map("a.b[2]", &map).values(),
            vec![&Value::Object(serde_json::Map::from_iter(vec![(
                "d".to_string(),
                Value::Object(serde_json::Map::from_iter(vec![(
                    "e".to_string(),
                    Value::Number(3.into())
                )]))
            )]))]
        );

        // select out of bound index from array
        assert!(get_value_from_json_map("a.b[3]", &map).check_is_empty());

        // select bad index from array
        assert!(get_value_from_json_map("a.b[z]", &map).check_is_empty());
    }

    #[test]
    fn test_get_deeply_nested_array_value_from_json_map() {
        let map = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"
            {
                "arr1": [
                    {
                        "arr2": [
                            {"a": 1, "b": 2}
                        ]
                    },
                    {
                        "arr2": [
                            {"a": 3, "b": 4},
                            {"a": 5, "b": 6}
                        ]
                    }
                ]
            }
            "#,
        )
        .unwrap();

        // extract and flatten all elements from arrays
        assert_eq!(
            get_value_from_json_map("arr1[].arr2[].a", &map).values(),
            vec![
                &Value::Number(1.into()),
                &Value::Number(3.into()),
                &Value::Number(5.into()),
            ]
        );
    }

    #[test]
    fn test_no_flatten_array_value_from_json_map() {
        let map = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"
            {
                "arr": [
                    { "a": [1, 2, 3] },
                    { "a": 4 },
                    { "b": 5 }
                ]
            }
            "#,
        )
        .unwrap();

        // extract and retain structure for arrays arrays
        assert_eq!(
            get_value_from_json_map("arr[].a", &map).values(),
            vec![
                &Value::Array(vec![
                    Value::Number(1.into()),
                    Value::Number(2.into()),
                    Value::Number(3.into()),
                ]),
                &Value::Number(4.into()),
            ]
        );

        // expect an array as leaf, ignore non arrays
        assert_eq!(
            get_value_from_json_map("arr[].a[]", &map).values(),
            vec![
                &Value::Number(1.into()),
                &Value::Number(2.into()),
                &Value::Number(3.into()),
            ]
        );
    }

    #[test]
    fn test_filter_json() {
        let map = serde_json::from_str::<serde_json::Map<String, Value>>(
            r#"
            {
                "a": {
                    "b": [
                        { "c": 1 },
                        { "c": 2 },
                        { "d": { "e": 3 } }
                    ]
                },
                "f": 3,
                "g": ["g0", "g1", "g2"]
            }
            "#,
        )
        .unwrap();

        let res = filter_json_values(&map, |path, _value| {
            path.starts_with("a.b[].c") || "a.b[].c".starts_with(path)
        });

        assert_eq!(
            res,
            serde_json::from_str::<serde_json::Map<String, Value>>(
                r#"
                {
                    "a": {
                        "b": [
                            { "c": 1 },
                            { "c": 2 },
                            {}
                        ]
                    }
                }
                "#,
            )
            .unwrap()
        );
    }
}
