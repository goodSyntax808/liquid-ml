//! A Schema module for managing the data types and row/column names of a
//! DataFrame.

use crate::dataframe::Schema;
use crate::error::LiquidError;
use sorer::schema::DataType;

/// The implementation of the Schema interface, which manages data types and
/// row/column names of a `DataFrame`
impl Schema {
    /// Constructs an empty Schema.
    pub fn new() -> Self {
        Schema {
            ..Default::default()
        }
    }

    /// Add a column with the given `data_type`, with an optional column name,
    /// to this Schema. Column names must be unique. If `col_name` is `Some`
    /// and the name already exists in this `Schema`, the column will not
    /// be added to this Schema and a `LiquidError::NameAlreadyExists` error
    /// will be returned.
    pub fn add_column(
        &mut self,
        data_type: DataType,
        col_name: Option<String>,
    ) -> Result<(), LiquidError> {
        match &col_name {
            Some(_name) => {
                if !self.col_names.contains(&col_name) {
                    self.schema.push(data_type);
                    self.col_names.push(col_name);
                    Ok(())
                } else {
                    Err(LiquidError::NameAlreadyExists)
                }
            }
            None => {
                self.schema.push(data_type);
                self.col_names.push(None);
                Ok(())
            }
        }
    }

    /// Gets the (optional) name of the column at the given `idx`.
    /// Returns a result that will `Error` if the `idx` is out of bounds.
    pub fn col_name(&self, idx: usize) -> Result<&Option<String>, LiquidError> {
        match self.col_names.get(idx) {
            Some(name) => Ok(name),
            None => Err(LiquidError::ColIndexOutOfBounds),
        }
    }

    /// Get the data type of the column at the given `idx`
    /// Returns a result that will `Error` if the `idx` is out of bounds.
    pub fn col_type(&self, idx: usize) -> Result<&DataType, LiquidError> {
        match self.schema.get(idx) {
            Some(data_type) => Ok(data_type),
            None => Err(LiquidError::ColIndexOutOfBounds),
        }
    }

    /// Given a column name, returns its index
    pub fn col_idx(&self, col_name: &str) -> Option<usize> {
        self.col_names.iter().position(|n| match n {
            Some(name) => col_name == name,
            None => false,
        })
    }

    /// The number of columns in this Schema.
    pub fn width(&self) -> usize {
        self.col_names.len()
    }

    fn char_to_data_type(c: char) -> DataType {
        match c {
            'B' => DataType::Bool,
            'I' => DataType::Int,
            'F' => DataType::Float,
            'S' => DataType::String,
            _ => panic!("Tried to make a bad Schema"),
        }
    }
}

impl From<&str> for Schema {
    /// Create a Schema from a `&str` of types. A string that contains
    /// characters other that `B`, `I`, `F`, or `S` will panic. Initializes
    /// the Column names to be `None`.
    ///
    /// | Character | DataType |
    /// |-----------|----------|
    /// | 'B'       | Bool     |
    /// | 'I'       | Int      |
    /// | 'F'       | Float    |
    /// | 'S'       | String   |
    fn from(types: &str) -> Self {
        let mut schema = Vec::new();
        let mut col_names = Vec::new();
        for c in types.chars() {
            schema.push(Schema::char_to_data_type(c));
            col_names.push(None);
        }
        Schema { schema, col_names }
    }
}

impl From<Vec<DataType>> for Schema {
    /// Create a Schema from a `Vec<DataType`
    fn from(types: Vec<DataType>) -> Self {
        let mut schema = Vec::new();
        let mut col_names = Vec::new();
        for t in types {
            schema.push(t);
            col_names.push(None);
        }
        Schema { schema, col_names }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_data_types() {
        let mut data_types = vec![];
        let mut s = Schema::from(data_types.clone());
        assert_eq!(s.width(), 0);
        data_types = vec![
            DataType::Int,
            DataType::Int,
            DataType::Float,
            DataType::Bool,
            DataType::String,
        ];
        s = Schema::from(data_types.clone());
        for (idx, data_type) in data_types.iter().enumerate() {
            assert_eq!(data_type, s.col_type(idx).unwrap());
        }
        assert_eq!(s.width(), data_types.len());
    }

    #[test]
    fn test_from_str() {
        let mut types_str = "";
        let mut s = Schema::from(types_str);
        assert_eq!(s.width(), 0);
        types_str = "IIFBS";
        s = Schema::from(types_str);
        let data_types = vec![
            DataType::Int,
            DataType::Int,
            DataType::Float,
            DataType::Bool,
            DataType::String,
        ];
        for (idx, data_type) in data_types.iter().enumerate() {
            assert_eq!(data_type, s.col_type(idx).unwrap());
        }
        assert_eq!(s.width(), data_types.len());
    }

    // add_column
    #[test]
    fn test_col_getters_setters() {
        // colummn getters/setters
        let mut s = Schema::new();
        assert_eq!(s.width(), 0);
        s.add_column(DataType::String, None).unwrap();
        assert_eq!(s.width(), 1);
        s.add_column(DataType::Int, Some(String::from("foo")))
            .unwrap();
        assert_eq!(s.width(), 2);
        assert_eq!(s.col_idx("foo"), Some(1));
        assert_eq!(s.col_name(0).unwrap(), &None);
    }
}
