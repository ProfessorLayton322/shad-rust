use std::{borrow::Cow, fmt};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ObjectId(i64);

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
        String => "TEXT",
        Bytes => "BLOB",
        Int64 => "BIGINT",
        Float64 => "REAL",
        Bool => "TINYINT",
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}

// TODO: you might want to add some code here.
