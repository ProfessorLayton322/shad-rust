use anyhow::{bail, ensure, Result};
use json::{object, JsonValue};

pub struct QueryData {
    pub collection: String,
    pub data: JsonValue,
}

pub enum Query {
    Insert(QueryData),
    Select(QueryData),
    Show,
}

pub fn parse_query(mut json_value: JsonValue) -> Result<Query> {
    ensure!(json_value.is_object());
    if json_value.has_key("show") {
        ensure!(
            json_value
                == object! {
                    show : "collections"
                }
        );
        return Ok(Query::Show);
    }
    ensure!(json_value.len() == 2);
    let collection: String = match json_value["collection"].take_string() {
        Some(string) => string,
        None => bail!("Collection name should be a string"),
    };
    if json_value.has_key("insert") {
        return Ok(Query::Insert(QueryData {
            collection,
            data: json_value["insert"].take(),
        }));
    }
    if json_value.has_key("select") {
        return Ok(Query::Select(QueryData {
            collection,
            data: json_value["select"].take(),
        }));
    }
    bail!("Invalid query")
}
