use std::{collections::HashMap, sync::Arc};

use rqlite_rs::{prelude::*, Column};

#[test]
fn integration_derive_from_row_named() {
    #[derive(FromRow)]
    struct Test {
        id: i32,
        name: String,
        #[allow(dead_code)]
        optional: Option<String>,
    }

    let mut column_names: HashMap<String, usize> = HashMap::new();
    let columns = vec![
        Column::new("id".to_string(), 0, "integer".to_string()),
        Column::new("name".to_string(), 1, "text".to_string()),
    ];

    columns.iter().enumerate().for_each(|(i, c)| {
        column_names.insert(c.name().to_string(), i);
    });

    let columns = Arc::new(columns);
    let column_names = Arc::new(column_names);

    let row = Row::new(
        &columns,
        &column_names,
        vec![serde_json::json!(1), serde_json::json!("test")].into_boxed_slice(),
    );

    let test = Test::from_row(row).unwrap();

    assert_eq!(test.id, 1);
    assert_eq!(test.name, "test");
}

#[test]
fn integration_derive_from_row_unnamed() {
    #[derive(FromRow)]
    struct Test(i32, String, #[allow(dead_code)] Option<String>);

    let columns = vec![
        Column::new("id".to_string(), 0, "integer".to_string()),
        Column::new("name".to_string(), 1, "text".to_string()),
    ];

    let mut column_names: HashMap<String, usize> = HashMap::new();
    columns.iter().enumerate().for_each(|(i, c)| {
        column_names.insert(c.name().to_string(), i);
    });

    let columns = Arc::new(columns);
    let column_names = Arc::new(column_names);

    let row = Row::new(
        &columns,
        &column_names,
        vec![serde_json::json!(1), serde_json::json!("test")].into_boxed_slice(),
    );

    let test = Test::from_row(row).unwrap();

    assert_eq!(test.0, 1);
    assert_eq!(test.1, "test");
}
