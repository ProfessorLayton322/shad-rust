use crate::ast::{parse_query, Query};
use crate::collection::Collection;
use anyhow::{bail, Result};
use json::JsonValue;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct Database {
    collections: HashMap<String, Collection>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            collections: HashMap::<String, Collection>::new(),
        }
    }

    pub fn exec(&mut self, stmt: impl AsRef<str>) -> Result<JsonValue> {
        let json_value = json::parse(stmt.as_ref())?;
        let query = parse_query(json_value)?;
        match query {
            Query::Insert(data) => self.insert(data.collection, data.data),
            Query::Select(data) => self.select(data.collection, data.data),
            Query::Show => bail!("I did not implement show query"),
        }
    }

    fn insert(&mut self, collection: String, data: JsonValue) -> Result<JsonValue> {
        self.collections
            .entry(collection)
            .or_insert_with(Collection::new)
            .insert(data)?;
        Ok(JsonValue::Null)
    }

    fn select(&mut self, collection: String, data: JsonValue) -> Result<JsonValue> {
        self.collections
            .entry(collection)
            .or_insert_with(Collection::new)
            .select(data)
    }
}
