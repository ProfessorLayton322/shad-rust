use crate::{
    data::DataType,
    object::{pull_id, pull_schema, Schema},
    ObjectId,
};

use rusqlite::types::Type;
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFound(Box<NotFoundError>),
    #[error(transparent)]
    UnexpectedType(Box<UnexpectedTypeError>),
    #[error(transparent)]
    MissingColumn(Box<MissingColumnError>),
    #[error("database is locked")]
    LockConflict,
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error>),
}

pub fn parse_column_name(msg: &str) -> Option<&'_ str> {
    let res = msg.strip_prefix("no such column: ");
    if res.is_some() {
        return res;
    }
    let schema = pull_schema();
    let pattern = format!("table {} has no column named ", schema.table_name);
    msg.strip_prefix(&pattern)
}

pub fn rusqltype_to_string(rusql_type: Type) -> String {
    match rusql_type {
        Type::Null => "Null",
        Type::Integer => "Integer",
        Type::Real => "Real",
        Type::Text => "Text",
        Type::Blob => "Blob",
    }
    .to_string()
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        let schema: &'static Schema = pull_schema();
        let id: ObjectId = pull_id();
        match err {
            rusqlite::Error::SqliteFailure(code, msg) => {
                if let rusqlite::ErrorCode::DatabaseBusy = code.code {
                    return Error::LockConflict;
                }
                let msg = msg.unwrap();
                let column_name = parse_column_name(&msg).unwrap();
                let field = schema.find_field(column_name).unwrap();
                Error::MissingColumn(Box::new(MissingColumnError {
                    type_name: schema.struct_name,
                    attr_name: field.attr_name,
                    table_name: schema.table_name,
                    column_name: field.column_name,
                }))
            }
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(Box::new(NotFoundError {
                object_id: id,
                type_name: schema.struct_name,
            })),
            rusqlite::Error::InvalidColumnType(_, column_name, rusqltype) => {
                let field = schema.find_field(&column_name).unwrap();
                Error::UnexpectedType(Box::new(UnexpectedTypeError {
                    type_name: schema.struct_name,
                    attr_name: field.attr_name,
                    table_name: schema.table_name,
                    column_name: field.column_name,
                    expected_type: field.data_type,
                    got_type: rusqltype_to_string(rusqltype),
                }))
            }
            _ => Error::Storage(Box::new(err)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("object is not found: type '{type_name}', id {object_id}")]
pub struct NotFoundError {
    pub object_id: ObjectId,
    pub type_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "invalid type for {type_name}::{attr_name}: expected equivalent of {expected_type:?}, \
    got {got_type} (table: {table_name}, column: {column_name})"
)]
pub struct UnexpectedTypeError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
    pub expected_type: DataType,
    pub got_type: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "missing a column for {type_name}::{attr_name} \
    (table: {table_name}, column: {column_name})"
)]
pub struct MissingColumnError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, Error>;
