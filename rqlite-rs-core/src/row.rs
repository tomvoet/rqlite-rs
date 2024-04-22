use std::{collections::HashMap, sync::Arc};

use serde_json::Value;

use crate::{column::Column, from_row::FromRow, IntoTypedError};

#[derive(Debug)]
pub struct Row {
    values: Box<[Value]>,
    columns: Arc<Vec<Column>>,
    column_names: Arc<HashMap<String, usize>>,
}

impl Row {
    pub fn new(
        columns: &Arc<Vec<Column>>,
        column_names: &Arc<HashMap<String, usize>>,
        values: Box<[Value]>,
    ) -> Row {
        Row {
            values,
            columns: Arc::clone(columns),
            column_names: Arc::clone(column_names),
        }
    }

    pub fn columns(&self) -> &Arc<Vec<Column>> {
        &self.columns
    }

    pub fn column_names(&self) -> &Arc<HashMap<String, usize>> {
        &self.column_names
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, name: &str) -> Result<T, IntoTypedError> {
        let index = self
            .column_names
            .get(name)
            .ok_or(IntoTypedError::ColumnNotFound)?;

        let value = self
            .values
            .get(*index)
            .ok_or(IntoTypedError::ValueNotFound)?;

        let value =
            serde_json::from_value(value.clone()).map_err(IntoTypedError::ConversionError)?;

        Ok(value)
    }

    pub fn get_opt<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Option<T>, IntoTypedError> {
        let Some(index) = self.column_names.get(name) else {
            return Ok(None);
        };

        // Throw error here, because if the column exists, the value should exist
        let value = self
            .values
            .get(*index)
            .ok_or(IntoTypedError::ValueNotFound)?;

        match value {
            Value::Null => Ok(None),
            _ => {
                let value = serde_json::from_value(value.clone())
                    .map_err(IntoTypedError::ConversionError)?;
                Ok(Some(value))
            }
        }
    }

    pub fn get_by_index<T: serde::de::DeserializeOwned>(
        &self,
        index: usize,
    ) -> Result<T, IntoTypedError> {
        let value = self
            .values
            .get(index)
            .ok_or(IntoTypedError::ValueNotFound)?;
        let value =
            serde_json::from_value(value.clone()).map_err(IntoTypedError::ConversionError)?;
        Ok(value)
    }

    pub fn get_by_index_opt<T: serde::de::DeserializeOwned>(
        &self,
        index: usize,
    ) -> Result<Option<T>, IntoTypedError> {
        let Some(value) = self.values.get(index) else {
            return Ok(None);
        };

        match value {
            Value::Null => Ok(None),
            _ => {
                let value = serde_json::from_value(value.clone())
                    .map_err(IntoTypedError::ConversionError)?;
                Ok(Some(value))
            }
        }
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn into_typed<T>(self) -> Result<T, IntoTypedError>
    where
        T: FromRow,
    {
        T::from_row(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_row() -> Row {
        use serde_json::json;

        let columns = Arc::new(vec![
            Column::new("id".to_string(), 0usize, "integer".to_string()),
            Column::new("name".to_string(), 1usize, "text".to_string()),
        ]);

        let column_names = Arc::new(
            columns
                .iter()
                .enumerate()
                .map(|(i, c)| (c.name().to_string(), i))
                .collect::<HashMap<String, usize>>(),
        );

        let values = vec![json!(1), json!("test")].into_boxed_slice();

        Row::new(&columns, &column_names, values)
    }

    #[test]
    fn unit_row_get() {
        let row = create_row();

        assert_eq!(row.get::<i32>("id").unwrap(), 1);
        assert_eq!(row.get::<String>("name").unwrap(), "test");
        assert!(matches!(
            row.get::<i32>("not_found").unwrap_err(),
            IntoTypedError::ColumnNotFound
        ));
    }

    #[test]
    fn unit_row_get_opt() {
        let row = create_row();

        assert_eq!(row.get_opt::<i32>("id").unwrap(), Some(1));
        assert_eq!(
            row.get_opt::<String>("name").unwrap(),
            Some("test".to_string())
        );
        assert_eq!(row.get_opt::<i32>("not_found").unwrap(), None);
    }

    #[test]
    fn unit_row_get_by_index() {
        let row = create_row();

        assert_eq!(row.get_by_index::<i32>(0).unwrap(), 1);
        assert_eq!(row.get_by_index::<String>(1).unwrap(), "test");
        assert!(matches!(
            row.get_by_index::<i32>(2).unwrap_err(),
            IntoTypedError::ValueNotFound
        ));
    }

    #[test]
    fn unit_row_get_by_index_opt() {
        let row = create_row();

        assert_eq!(row.get_by_index_opt::<i32>(0).unwrap(), Some(1));
        assert_eq!(
            row.get_by_index_opt::<String>(1).unwrap(),
            Some("test".to_string())
        );
        assert_eq!(row.get_by_index_opt::<i32>(2).unwrap(), None);
    }

    #[test]
    fn unit_row_columns() {
        let row = create_row();

        assert_eq!(row.columns().len(), 2);
        assert_eq!(row.columns()[0].name(), "id");
        assert_eq!(row.columns()[1].name(), "name");
    }

    #[test]
    fn unit_row_column_names() {
        let row = create_row();

        assert_eq!(row.column_names().len(), 2);
        assert_eq!(row.column_names()["id"], 0);
        assert_eq!(row.column_names()["name"], 1);
    }

    #[test]
    fn unit_row_values() {
        let row = create_row();

        assert_eq!(row.values().len(), 2);
        assert_eq!(row.values()[0], serde_json::json!(1));
        assert_eq!(row.values()[1], serde_json::json!("test"));
    }

    #[test]
    fn unit_row_len() {
        let row = create_row();

        assert_eq!(row.len(), 2);
    }

    #[test]
    fn unit_row_is_empty() {
        let row = create_row();

        assert!(!row.is_empty());
    }

    #[test]
    fn unit_row_into_typed_tuple() {
        let row = create_row();

        let (id, name) = row.into_typed::<(i32, String)>().unwrap();

        assert_eq!(id, 1);
        assert_eq!(name, "test");
    }

    #[test]
    fn unit_row_into_typed_struct() {
        struct Test {
            id: i32,
            name: String,
        }

        impl FromRow for Test {
            fn from_row(row: Row) -> Result<Self, IntoTypedError> {
                Ok(Test {
                    id: row.get("id")?,
                    name: row.get("name")?,
                })
            }
        }

        let row = create_row();

        let test = row.into_typed::<Test>().unwrap();

        assert_eq!(test.id, 1);
        assert_eq!(test.name, "test");
    }
}
