use proc_macro::TokenStream;
use syn::{parse_macro_input, Attribute, Data, DeriveInput};

#[proc_macro_derive(Object, attributes(table_name, column_name))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = ast.ident.to_string();

    fn try_get_attr(attrs: &[Attribute]) -> Option<String> {
        let attr = attrs.iter().next()?;
        let token = attr.tokens.clone().into_iter().next()?;
        let smth = token.to_string();
        if smth.len() <= 4 {
            return None;
        }
        Some(String::from(smth.as_str().strip_prefix("(\"")?.strip_suffix("\")")?))
    }

    let table_name = match try_get_attr(&ast.attrs) {
        Some(s) => s,
        None => struct_name.clone(),
    };
    let Data::Struct(some_struct) = ast.data else {
        panic!("Lol");
    };
    let syn::Fields::Named(fields) = some_struct.fields else {
        let implementation = format!("
    impl Object for {0} {{
        const SCHEMA : orm::object::Schema = orm::object::Schema {{
            struct_name: \"{1}\",
            table_name: \"{2}\",
            fields: &[],
        }};

        fn to_row(&self) -> orm::storage::Row {{
            vec![]
        }}

        fn from_row(row : &orm::storage::RowSlice) -> Self {{
            Self {{}}
        }}
    }}",
        struct_name,
        struct_name,
        table_name);
        return implementation.parse().unwrap();
    };
    let mut keys = Vec::new();
    let mut types = Vec::new();
    let mut columns = Vec::new();
    for field in fields.named.iter() {
        let name = field.ident.as_ref().unwrap().to_string();
        columns.push(match try_get_attr(&field.attrs) {
            None => name.clone(),
            Some(s) => s,
        });
        keys.push(name);
        let syn::Type::Path(path) = &field.ty else {
            panic!();
        };
        types.push(path.path.clone().segments.into_iter().next().unwrap().ident.to_string());
    }
    fn parse_type(t: &str) -> &'static str {
        match t {
            "String" => "String",
            "Vec" => "Bytes",
            "i64" => "Int64",
            "f64" => "Float64",
            "bool" => "Bool",
            _ => panic!("shit"),
        }
    }
    let implementation = format!("
impl Object for {0} {{
    const SCHEMA : orm::object::Schema = orm::object::Schema {{
        struct_name: \"{1}\",
        table_name: \"{2}\",
        fields: &[
            {3}
        ],
    }};

    fn to_row(&self) -> orm::storage::Row {{
        vec![
            {4}
        ]
    }}

    fn from_row(row: &orm::storage::RowSlice) -> Self {{
        let mut values = row.iter();
        Self {{
            {5}
        }}
    }}
}}
    ",
    struct_name,
    struct_name,
    table_name,
    keys.iter().zip(types.iter().map(|x| parse_type(x))).zip(columns.iter()).map(|((name, t), column)| {
        format!("
            orm::object::FieldInfo {{
                column_name: \"{0}\",
                attr_name: \"{1}\",
                data_type: orm::data::DataType::{2},
            }},",
        column,
        name,
        t
        )
    }).collect::<Vec<String>>().join(""),
    keys.iter().map(|name| {
        format!("
                orm::data::Value::from(&self.{0}),", name)
    }).collect::<Vec<String>>().join(""),
    keys.iter().map(|name| {
        format!("
            {0}: values.next().unwrap().into(),", name)
    }).collect::<Vec<String>>().join("")
    );
    implementation.parse().unwrap()
}

