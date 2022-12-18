use crate::{
    data::{datatype_to_sql, DataType, Value},
    error::Result,
    object::Schema,
    ObjectId,
};

use rusqlite::params;
use rusqlite::ToSql;

use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////

pub type Row<'a> = Vec<Value<'a>>;
pub type RowSlice<'a> = [Value<'a>];

pub fn row_to_params<'a>(row: &'a RowSlice<'a>) -> Vec<&'a dyn ToSql> {
    row.iter().map(|x| x as &dyn ToSql).collect()
}

////////////////////////////////////////////////////////////////////////////////

pub(crate) trait StorageTransaction {
    fn table_exists(&self, table: &str) -> Result<bool>;
    fn create_table(&self, schema: &Schema) -> Result<()>;

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId>;
    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()>;
    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>>;
    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()>;

    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}

impl<'a> StorageTransaction for rusqlite::Transaction<'a> {
    fn table_exists(&self, table: &str) -> Result<bool> {
        let mut stmt = self.prepare("SELECT 1 FROM sqlite_master WHERE name == (?1)")?;
        let mut weird_iter = stmt
            .query_map(params![table], |row| {
                let ans: bool = row.get(0)?;
                Ok(ans)
            })
            .unwrap();
        Ok(weird_iter.next().is_some())
    }

    fn create_table(&self, schema: &Schema) -> Result<()> {
        let query = format!(
            "CREATE TABLE {0} (id INTEGER PRIMARY KEY AUTOINCREMENT{1})",
            schema.table_name,
            schema
                .fields
                .iter()
                .map(|field| {
                    format!(
                        ", {0} {1}",
                        field.column_name,
                        datatype_to_sql(field.data_type)
                    )
                })
                .collect::<String>()
        );
        self.execute(&query, params![])?;
        Ok(())
    }

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId> {
        let query = match schema.fields.len() {
            0 => format!("INSERT INTO {0} DEFAULT VALUES", schema.table_name),
            _ => format!(
                "INSERT INTO {0}({1}) VALUES ({2})",
                schema.table_name,
                schema
                    .fields
                    .iter()
                    .map(|field| field.column_name)
                    .collect::<Vec<&str>>()
                    .join(", "),
                vec!["?"; schema.fields.len()].join(", ")
            ),
        };
        let mut stmt = self.prepare(&query)?;

        let content = row_to_params(row);
        stmt.execute(&*content)?;

        Ok(ObjectId(self.last_insert_rowid()))
    }

    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()> {
        let query = format!(
            "UPDATE {0} SET {1} WHERE id = {2}",
            schema.table_name,
            schema
                .fields
                .iter()
                .map(|field| { format!("{0} = ?", field.column_name) })
                .collect::<Vec<String>>()
                .join(", "),
            id.0
        );
        let mut stmt = self.prepare(&query)?;

        let content = row_to_params(row);
        stmt.execute(&*content)?;
        Ok(())
    }

    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>> {
        let query = match schema.fields.len() {
            0 => format!("SELECT * FROM {0} WHERE id = {1}", schema.table_name, id.0),
            _ => format!(
                "SELECT {0} FROM {1} WHERE id = {2}",
                schema
                    .fields
                    .iter()
                    .map(|field| field.column_name)
                    .collect::<Vec<&str>>()
                    .join(", "),
                schema.table_name,
                id.0
            ),
        };
        let mut stmt = self.prepare(&query)?;
        let pull = stmt.query_row(
            params![],
            |row: &rusqlite::Row<'_>| -> rusqlite::Result<Row<'static>> {
                schema
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| match field.data_type {
                        DataType::String => {
                            let s: String = row.get(i)?;
                            Ok(Value::String(Cow::from(s)))
                        }
                        DataType::Bytes => {
                            let v: Vec<u8> = row.get(i)?;
                            Ok(Value::Bytes(Cow::from(v)))
                        }
                        DataType::Int64 => Ok(Value::Int64(row.get(i)?)),
                        DataType::Float64 => Ok(Value::Float64(row.get(i)?)),
                        DataType::Bool => Ok(Value::Bool(row.get(i)?)),
                    })
                    .collect()
            },
        );
        Ok(pull?)
    }

    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()> {
        let query = format!("DELETE FROM {0} WHERE id == {1}", schema.table_name, id.0);
        self.execute(&query, params![])?;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        self.execute("COMMIT", params![])?;
        Ok(())
    }

    fn rollback(&self) -> Result<()> {
        self.execute("ROLLBACK", params![])?;
        Ok(())
    }
}
