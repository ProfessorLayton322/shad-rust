use crate::{
    data::{DataType, ObjectId},
    storage::Row,
    storage::RowSlice,
};

use std::any::Any;

use std::cell::RefCell;

////////////////////////////////////////////////////////////////////////////////

pub trait Object: Any {
    const SCHEMA: Schema;
    fn to_row(&self) -> Row;
    fn from_row(row: &RowSlice) -> Self;
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FieldInfo {
    pub column_name: &'static str,
    pub attr_name: &'static str,
    pub data_type: DataType,
}

#[derive(Debug, Copy, Clone)]
pub struct Schema {
    pub struct_name: &'static str,
    pub table_name: &'static str,
    pub fields: &'static [FieldInfo],
}

impl Schema {
    pub fn find_field(&self, column_name: &str) -> Option<&'static FieldInfo> {
        self.fields
            .iter()
            .find(|&field| field.column_name == column_name)
    }
}

thread_local!(
    static LAST_SCHEMA: RefCell<Option<&'static Schema>> = RefCell::new(None);
    static LAST_ID: RefCell<ObjectId> = RefCell::new(ObjectId(-1));
);

pub fn fetch_schema(schema: &'static Schema) {
    LAST_SCHEMA.with(|cell| *cell.borrow_mut() = Some(schema));
}

pub fn pull_schema() -> &'static Schema {
    LAST_SCHEMA.with(|cell| cell.borrow().unwrap())
}

pub fn fetch_id(id: ObjectId) {
    LAST_ID.with(|cell| *cell.borrow_mut() = id);
}

pub fn pull_id() -> ObjectId {
    LAST_ID.with(|cell| *cell.borrow())
}

pub trait Store: Any {
    fn get_row(&self) -> Row;
    fn get_schema(&self) -> &'static Schema;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Object + Sized> Store for T {
    fn get_row(&self) -> Row {
        self.to_row()
    }

    fn get_schema(&self) -> &'static Schema {
        fetch_schema(&T::SCHEMA);
        &T::SCHEMA
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
