use rusqlite::ToSql;
use std::convert::From;
use std::{borrow::Cow, fmt};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ObjectId(pub i64);

impl ObjectId {
    pub fn into_i64(&self) -> i64 {
        self.0
    }
}

impl From<i64> for ObjectId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.0)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    String,
    Bytes,
    Int64,
    Float64,
    Bool,
}

pub fn datatype_to_sql(datatype: DataType) -> &'static str {
    match datatype {
        DataType::String => "TEXT",
        DataType::Bytes => "BLOB",
        DataType::Int64 => "BIGINT",
        DataType::Float64 => "REAL",
        DataType::Bool => "TINYINT",
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}

impl ToSql for Value<'_> {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>, rusqlite::Error> {
        match self {
            Value::String(s) => s.as_ref().to_sql(),
            Value::Bytes(b) => b.as_ref().to_sql(),
            Value::Int64(i) => i.to_sql(),
            Value::Float64(f) => f.to_sql(),
            Value::Bool(b) => b.to_sql(),
        }
    }
}

impl<'a> From<&'a String> for Value<'a> {
    fn from(s: &'a String) -> Self {
        Value::String(Cow::from(s))
    }
}

impl<'a> From<&'a Vec<u8>> for Value<'a> {
    fn from(v: &'a Vec<u8>) -> Self {
        Value::Bytes(Cow::from(v))
    }
}

impl From<&i64> for Value<'_> {
    fn from(v: &i64) -> Self {
        Value::Int64(*v)
    }
}

impl From<&f64> for Value<'_> {
    fn from(v: &f64) -> Self {
        Value::Float64(*v)
    }
}

impl From<&bool> for Value<'_> {
    fn from(v: &bool) -> Self {
        Value::Bool(*v)
    }
}

impl From<&Value<'_>> for String {
    fn from(value: &Value<'_>) -> String {
        let Value::String(s) = value else {
            panic!("Type mismatch");
        };
        String::from(s.as_ref())
    }
}

impl From<&Value<'_>> for Vec<u8> {
    fn from(value: &Value<'_>) -> Vec<u8> {
        let Value::Bytes(b) = value else {
            panic!("Type mismatch");
        };
        Vec::<u8>::from(b.as_ref())
    }
}

impl From<&Value<'_>> for i64 {
    fn from(value: &Value<'_>) -> i64 {
        let Value::Int64(i) = value else {
            panic!("Type mismatch");
        };
        *i
    }
}

impl From<&Value<'_>> for f64 {
    fn from(value: &Value<'_>) -> f64 {
        let Value::Float64(f) = value else {
            panic!("Type mismatch");
        };
        *f
    }
}

impl From<&Value<'_>> for bool {
    fn from(value: &Value<'_>) -> bool {
        let Value::Bool(b) = value else {
            panic!("Type mismatch");
        };
        *b
    }
}
