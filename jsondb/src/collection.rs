use crate::data::{check_predicate, inner_to_json, is_valid_insert, json_to_inner, Value};
use anyhow::{ensure, Result};
use json::JsonValue;

pub struct Collection {
    content: Vec<Value>,
}

impl Collection {
    pub fn new() -> Self {
        Self {
            content: Vec::<Value>::new(),
        }
    }

    pub fn insert(&mut self, data: JsonValue) -> Result<()> {
        let inner = json_to_inner(data)?;
        println!("{:?}", inner);
        ensure!(is_valid_insert(&inner));
        self.content.push(inner);
        Ok(())
    }

    pub fn select(&self, data: JsonValue) -> Result<JsonValue> {
        let inner = json_to_inner(data)?;
        let mut answer = JsonValue::new_array();
        for document in self.content.iter() {
            let mut temp = inner.clone();
            if check_predicate(document, &mut temp)? {
                answer.push(inner_to_json(document)?)?;
            }
        }
        Ok(answer)
    }
}
