//! Structs and functions for working with rows of data in a `DataFrame`.

use crate::dataframe::fielder::Fielder;
use crate::dataframe::{Row, Schema};
use crate::error::LiquidError;
use sorer::dataframe::Data;
use sorer::schema::DataType;

/// Functions for creating, mutating, and getting data from `Row`s.
impl Row {
    /// Constructs a new `Row` with the given `Schema` and fills it with
    /// `Null` values.
    pub fn new(schema: &Schema) -> Self {
        let mut data: Vec<Data> = Vec::new();
        for _ in &schema.schema {
            data.push(Data::Null);
        }

        Row {
            schema: schema.schema.clone(),
            data,
            idx: None,
        }
    }

    /// Sets the field in this `Row` at the given `col_idx` to have the given
    /// `data`. The `DataType` at the `col_idx` must be an `Int` or it will
    /// return a `TypeMismatch` error.
    pub fn set_int(
        &mut self,
        col_idx: usize,
        data: i64,
    ) -> Result<(), LiquidError> {
        match self.schema.get(col_idx) {
            Some(DataType::Int) => Ok(match self.data.get(col_idx).unwrap() {
                Data::Null | Data::Int(_) => {
                    *self.data.get_mut(col_idx).unwrap() = Data::Int(data)
                }
                _ => panic!("Something is horribly wrong"),
            }),
            None => Err(LiquidError::ColIndexOutOfBounds),
            _ => Err(LiquidError::TypeMismatch),
        }
    }

    /// Sets the field in this `Row` at the given `col_idx` to have the given
    /// `data`. The `DataType` at the `col_idx` must be a `Float` or it will
    /// return a `TypeMismatch` error.
    pub fn set_float(
        &mut self,
        col_idx: usize,
        data: f64,
    ) -> Result<(), LiquidError> {
        match self.schema.get(col_idx) {
            Some(DataType::Float) => {
                Ok(match self.data.get(col_idx).unwrap() {
                    Data::Null | Data::Float(_) => {
                        *self.data.get_mut(col_idx).unwrap() = Data::Float(data)
                    }
                    _ => panic!("Something is horribly wrong"),
                })
            }
            None => Err(LiquidError::ColIndexOutOfBounds),
            _ => Err(LiquidError::TypeMismatch),
        }
    }

    /// Sets the field in this `Row` at the given `col_idx` to have the given
    /// `data`. The `DataType` at the `col_idx` must be a `Bool` or it will
    /// return a `TypeMismatch` error.
    pub fn set_bool(
        &mut self,
        col_idx: usize,
        data: bool,
    ) -> Result<(), LiquidError> {
        match self.schema.get(col_idx) {
            Some(DataType::Bool) => Ok(match self.data.get(col_idx).unwrap() {
                Data::Null | Data::Bool(_) => {
                    *self.data.get_mut(col_idx).unwrap() = Data::Bool(data)
                }
                _ => panic!("Something is horribly wrong"),
            }),
            None => Err(LiquidError::ColIndexOutOfBounds),
            _ => Err(LiquidError::TypeMismatch),
        }
    }

    /// Sets the field in this `Row` at the given `col_idx` to have the given
    /// `data`. The `DataType` at the `col_idx` must be a `String` or it will
    /// return a `TypeMismatch` error.
    pub fn set_string(
        &mut self,
        col_idx: usize,
        data: String,
    ) -> Result<(), LiquidError> {
        match self.schema.get(col_idx) {
            Some(DataType::String) => {
                Ok(match self.data.get(col_idx).unwrap() {
                    Data::Null | Data::String(_) => {
                        *self.data.get_mut(col_idx).unwrap() =
                            Data::String(data)
                    }
                    _ => panic!("Something is horribly wrong"),
                })
            }
            None => Err(LiquidError::ColIndexOutOfBounds),
            _ => Err(LiquidError::TypeMismatch),
        }
    }

    /// Sets the field in this `Row` at the given `col_idx` to be `Null`.
    pub fn set_null(&mut self, col_idx: usize) -> Result<(), LiquidError> {
        match self.data.get(col_idx) {
            Some(_) => {
                *self.data.get_mut(col_idx).unwrap() = Data::Null;
                Ok(())
            }
            _ => Err(LiquidError::ColIndexOutOfBounds),
        }
    }

    /// Set the row offset in the daaframe for this `Row`.
    pub fn set_idx(&mut self, idx: usize) {
        self.idx = Some(idx);
    }

    /// Get the current index of this `Row`.
    pub fn get_idx(&self) -> Result<usize, LiquidError> {
        match self.idx {
            Some(index) => Ok(index),
            None => Err(LiquidError::NotSet),
        }
    }

    /// Get a reference of the boxed value at the given `idx`.
    pub fn get(&self, idx: usize) -> Result<&Data, LiquidError> {
        match self.data.get(idx) {
            Some(d) => Ok(d),
            None => Err(LiquidError::ColIndexOutOfBounds),
        }
    }

    /// Get the number of columns in this `Row`.
    pub fn width(&self) -> usize {
        self.data.len()
    }

    /// Get the `DataType` of the `Column` at the given `idx`.
    pub fn col_type(&self, idx: usize) -> Result<&DataType, LiquidError> {
        match self.schema.get(idx) {
            Some(d) => Ok(d),
            None => Err(LiquidError::ColIndexOutOfBounds),
        }
    }

    /// Accept a visitor for this row that vists all the elements in this `Row`.
    pub fn accept<T: Fielder>(&self, f: &mut T) -> Result<(), LiquidError> {
        let idx = match self.get_idx() {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        f.start(idx);

        for data in &self.data {
            match data {
                Data::Int(d) => f.visit_int(*d),
                Data::Bool(d) => f.visit_bool(*d),
                Data::Float(d) => f.visit_float(*d),
                Data::String(d) => f.visit_string(&d),
                Data::Null => f.visit_null(),
            }
        }

        f.done();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataframe::fielder::Fielder;
    use crate::dataframe::Schema;

    struct TestFielder {
        pub num_null: usize,
        pub num_ints: usize,
        pub num_bools: usize,
        pub num_floats: usize,
        pub num_strings: usize,
        pub start_idx: usize,
    }

    impl Fielder for TestFielder {
        fn start(&mut self, starting_row_index: usize) {
            self.start_idx = starting_row_index;
        }

        fn visit_bool(&mut self, _b: bool) {
            self.num_bools += 1;
        }

        fn visit_float(&mut self, _f: f64) {
            self.num_floats += 1;
        }

        fn visit_int(&mut self, _i: i64) {
            self.num_ints += 1;
        }

        fn visit_string(&mut self, _s: &String) {
            self.num_strings += 1;
        }

        fn visit_null(&mut self) {
            self.num_null += 1;
        }

        fn done(&mut self) {}
    }

    fn init() -> (Vec<DataType>, Schema, Row) {
        let data_types = vec![
            DataType::Int,
            DataType::Bool,
            DataType::Float,
            DataType::String,
        ];
        let s = Schema::from(data_types.clone());
        let r = Row::new(&s);

        (data_types, s, r)
    }

    #[test]
    fn test_accept() {
        let (_data_types, _s, mut r) = init();
        r.set_int(0, 42).unwrap();
        r.set_bool(1, true).unwrap();
        r.set_float(2, 420.69).unwrap();
        r.set_string(3, String::from("Finally a sane language"))
            .unwrap();
        let mut f = TestFielder {
            num_null: 0,
            num_ints: 0,
            num_bools: 0,
            num_floats: 0,
            num_strings: 0,
            start_idx: 1,
        };
        r.set_idx(1);
        r.accept(&mut f).unwrap();
        assert_eq!(f.num_null, 0);
        assert_eq!(f.num_ints, 1);
        assert_eq!(f.num_bools, 1);
        assert_eq!(f.num_floats, 1);
        assert_eq!(f.num_strings, 1);
    }

    #[test]
    fn test_width() {
        let mut s = Schema::new();
        let r = Row::new(&s);
        assert_eq!(r.width(), 0);
        s.add_column(DataType::Int, None).unwrap();
        assert_eq!(r.width(), 0);
        let r2 = Row::new(&s);
        assert_eq!(r2.width(), 1);
    }

    #[test]
    fn test_col_type() {
        let (data_types, _s, r) = init();
        for (idx, data_type) in data_types.iter().enumerate() {
            assert_eq!(data_type, r.col_type(idx).unwrap());
        }
    }

    #[test]
    fn test_get_set_idx() {
        let (_data_types, _s, mut r) = init();
        assert_eq!(r.get_idx().is_err(), true);
        r.set_idx(0);
        assert_eq!(r.get_idx().unwrap(), 0);
    }

    #[test]
    fn test_getters_and_setters() {
        let (_data_types, _s, mut r) = init();

        for d in r.data.iter() {
            assert_eq!(&Data::Null, d);
        }

        r.set_int(0, 42).unwrap();
        assert_eq!(&Data::Int(42), r.get(0).unwrap());
        r.set_bool(1, false).unwrap();
        assert_eq!(&Data::Bool(false), r.get(1).unwrap());
        r.set_float(2, 3.14).unwrap();
        assert_eq!(&Data::Float(3.14), r.get(2).unwrap());
        r.set_string(3, String::from("foo")).unwrap();
        assert_eq!(&Data::String(String::from("foo")), r.get(3).unwrap());

        r.set_null(3).unwrap();
        assert_eq!(&Data::Null, r.get(3).unwrap());
    }
}