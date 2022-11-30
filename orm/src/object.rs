use crate::{data::DataType, data::datatype_to_sql, storage::Row};

use std::any::Any;

////////////////////////////////////////////////////////////////////////////////

pub trait Object: Any {
    /*
    fn to_row(&self) -> Row;
    fn from_row(row: &Row) -> Self;
    */
    fn get_schema() -> Schema;
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FieldInfo {
    pub column_name: &'static str,
    pub attr_name: &'static str,
    pub data_type: DataType,
}

#[derive(Debug)]
pub struct Schema {
    pub struct_name: &'static str,
    pub table_name: &'static str,
    pub fields: Vec<FieldInfo>,
}

impl Schema {
    pub fn create_table_query(&self) -> String {
        let mut ans = format!("CREATE TABLE {0} (id INTEGER PRIMARY KEY AUTOINCREMENT", self.table_name);
        for field in self.fields.iter() {
            ans.push_str(format!(",{0} {1}", field.column_name, datatype_to_sql(field.data_type)).as_str());
        }
        ans.push(')');
        ans
    }
}
