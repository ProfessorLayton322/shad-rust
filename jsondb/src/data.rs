use anyhow::{bail, ensure, Result};
use json::JsonValue;
use std::cmp::{min, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::string::String;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Dict(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Null,
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Dict(dict1), Value::Dict(dict2)) => {
                if dict1 == dict2 {
                    return Some(Ordering::Equal);
                }
                None
            }
            (Value::Array(arr1), Value::Array(arr2)) => {
                for i in 0..min(arr1.len(), arr2.len()) {
                    match arr1
                        .get(i)
                        .unwrap()
                        .partial_cmp(arr2.get(i).unwrap())
                        .unwrap()
                    {
                        Ordering::Equal => {}
                        Ordering::Less => return Some(Ordering::Less),
                        Ordering::Greater => return Some(Ordering::Greater),
                    }
                }
                arr1.len().partial_cmp(&arr2.len())
                /*
                if arr1.len() < arr2.len() {
                    return Some(Ordering::Less);
                } else if arr1.len() > arr2.len() {
                    return Some(Ordering::Greater);
                }
                Some(Ordering::Equal)
                */
            }
            (Value::String(val1), Value::String(val2)) => val1.partial_cmp(val2),
            (Value::Int(val1), Value::Int(val2)) => val1.partial_cmp(val2),
            (Value::Float(val1), Value::Float(val2)) => val1.partial_cmp(val2),
            (Value::Bool(val1), Value::Bool(val2)) => val1.partial_cmp(val2),
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            _ => None,
        }
    }
}

pub fn json_to_inner(json_value: JsonValue) -> Result<Value> {
    println!("{:?}", json_value);
    match json_value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::String(string) => Ok(Value::String(string)),
        JsonValue::Short(short_string) => Ok(Value::String(short_string.to_string())),
        JsonValue::Number(_) => {
            if let Some(value) = json_value.as_f32() {
                return Ok(Value::Float(value));
            }
            if let Some(value) = json_value.as_i32() {
                return Ok(Value::Int(value));
            }
            bail!("Invalid integral type");
        }
        JsonValue::Boolean(value) => Ok(Value::Bool(value)),
        JsonValue::Object(mut object) => {
            let mut answer = HashMap::<String, Value>::new();
            for pair in object.iter_mut() {
                let key = pair.0.to_string();
                let value = json_to_inner(pair.1.take())?;
                ensure!(answer.insert(key, value).is_none());
            }
            Ok(Value::Dict(answer))
        }
        JsonValue::Array(mut array) => {
            let mut answer = Vec::<Value>::new();
            for value in array.iter_mut() {
                let inner_value = json_to_inner(value.take())?;
                answer.push(inner_value);
            }
            Ok(Value::Array(answer))
        }
    }
}

pub fn inner_to_json(inner: &Value) -> Result<JsonValue> {
    match inner {
        Value::Dict(dict) => {
            let mut answer = JsonValue::new_object();
            for (key, value) in dict {
                let json_value = inner_to_json(value)?;
                answer.insert(key.as_str(), json_value)?;
            }
            Ok(answer)
        }
        Value::Array(array) => {
            let mut answer = JsonValue::new_array();
            for value in array {
                let json_value = inner_to_json(value)?;
                answer.push(json_value)?;
            }
            Ok(answer)
        }
        Value::String(string) => Ok(JsonValue::from(string.as_str())),
        Value::Int(value) => Ok(JsonValue::from(*value)),
        Value::Float(value) => Ok(JsonValue::from(*value)),
        Value::Bool(value) => Ok(JsonValue::from(*value)),
        Value::Null => Ok(JsonValue::Null),
    }
}

pub fn is_valid_insert(inner: &Value) -> bool {
    match inner {
        Value::Dict(dict) => {
            for key in dict.keys() {
                if key.starts_with('$') {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub enum Predicate {
    Empty,
    Flat(Value),
    Name((String, Value)),
    Gt(Value),
    Le(Value),
    Eq(Value),
    Or(Vec<Value>),
    And(Vec<Value>),
    In(Vec<Value>),
}

pub fn inner_to_predicate(inner: &mut Value) -> Result<Predicate> {
    match inner {
        Value::Dict(dict) => {
            ensure!(dict.len() < 2);
            if dict.is_empty() {
                return Ok(Predicate::Empty);
            }
            let name: String = dict.keys().next().unwrap().clone();
            let value = dict.remove(&name).unwrap();
            match name.as_str() {
                "$lt" => Ok(Predicate::Le(value)),
                "$gt" => Ok(Predicate::Gt(value)),
                "$eq" => Ok(Predicate::Eq(value)),
                "$in" => match value {
                    Value::Array(arr) => Ok(Predicate::In(arr)),
                    _ => bail!("$in should be followed by a list"),
                },
                "$or" => match value {
                    Value::Array(arr) => Ok(Predicate::Or(arr)),
                    _ => bail!("$or should be followed by a list"),
                },
                "$and" => match value {
                    Value::Array(arr) => Ok(Predicate::And(arr)),
                    _ => bail!("$and should be followed by a list"),
                },
                _ => {
                    ensure!(!name.starts_with('$'));
                    Ok(Predicate::Name((name, value)))
                }
            }
        }
        _ => Ok(Predicate::Flat(inner.clone())),
    }
}

pub fn check_predicate(object: &Value, predicate_value: &mut Value) -> Result<bool> {
    match inner_to_predicate(predicate_value)? {
        Predicate::Empty => Ok(true),
        Predicate::Name((name, mut condition)) => {
            if let Value::Dict(dict) = object {
                match dict.get(&name) {
                    None => check_predicate(&Value::Null, &mut condition),
                    Some(key_value) => check_predicate(key_value, &mut condition),
                }
            } else {
                bail!("Incorrect query");
            }
        }
        Predicate::Gt(value) => Ok(object > &value),
        Predicate::Le(value) => Ok(object < &value),
        Predicate::Eq(value) => Ok(object == &value),
        Predicate::Or(mut arr) => {
            for value in arr.iter_mut() {
                if check_predicate(object, value)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Predicate::And(mut arr) => {
            for value in arr.iter_mut() {
                if !check_predicate(object, value)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Predicate::In(arr) => {
            for value in arr.iter() {
                if object == value {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Predicate::Flat(value) => Ok(object == &value),
    }
}
