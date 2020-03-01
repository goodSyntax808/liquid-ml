//! A module for creating and manipulating
//! [`DataFrame`s](::crate::dataframe::DataFrame). A `DataFrame` can be created
//! from a [`SoR`](sorer) file, by adding [`Column`s](sorer::dataframe::Column),
//! or by adding [`Row`s](::crate::row::Row). You may implement the
//! [`Rower`](::crate::rower::Rower) trait to perform `map` operations on
//! a `DataFrame`.

use crate::error::DFError;
use crate::row::Row;
use crate::rower::Rower;
use crate::schema::Schema;
use num_cpus;
use sorer::dataframe::{from_file, Column, Data};
use sorer::schema::{infer_schema_from_file, DataType};
use std::thread;

/// Represents a DataFrame which contains
/// [columnar](sorer::dataframe::Column) `Data` and a
/// [Schema](::crate::schema::Schema).
pub struct DataFrame {
    /// The [Schema](::crate::schema::Schema) of this DataFrame
    pub schema: Schema,
    /// The [columnar](::crate::dataframe::Column) data of this DataFrame.
    pub data: Vec<Column>,
    /// Number of threads for this computer
    pub n_threads: usize,
}

const IDX_OUT_OF_BOUNDS: fn() = || panic!("Index out of bounds");

/// Traits defining a `DataFrame` inspired by those used in `pandas` and `R`.
impl DataFrame {
    /// Creates a new `DataFrame` from the given file, only reads `len` bytes
    /// starting at the given byte offset `from`.
    pub fn from_sor(file_name: String, from: usize, len: usize) -> Self {
        let schema = Schema::from(infer_schema_from_file(file_name.clone()));
        let data = from_file(file_name, schema.schema.clone(), from, len);
        DataFrame {
            schema,
            data,
            n_threads: num_cpus::get(),
        }
    }

    /// Creates an empty `DataFrame` from the given
    /// [`Schema`](::crate::schema::Schema).
    pub fn new(s: Schema) -> Self {
        let mut data = Vec::new();
        for data_type in &s.schema {
            match data_type {
                DataType::Bool => data.push(Column::Bool(Vec::new())),
                DataType::Int => data.push(Column::Int(Vec::new())),
                DataType::Float => data.push(Column::Float(Vec::new())),
                DataType::String => data.push(Column::String(Vec::new())),
            }
        }
        let schema = Schema {
            schema: s.schema.clone(),
            col_names: s.col_names.clone(),
            row_names: Vec::new(),
        };

        DataFrame {
            schema,
            data,
            n_threads: num_cpus::get(),
        }
    }

    /// Obtains a reference to this `DataFrame`s schema.
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }

    /// Adds a [`Column`](sorer::dataframe::Column) to this `DataFrame`.
    pub fn add_column(&mut self, col: Column, name: Option<String>) {
        match col {
            Column::Int(_) => self.schema.add_column(DataType::Int, name),
            Column::Bool(_) => self.schema.add_column(DataType::Bool, name),
            Column::Float(_) => self.schema.add_column(DataType::Float, name),
            Column::String(_) => self.schema.add_column(DataType::String, name),
        };
    }

    /// Get the [`Data`](sorer::dataframe::Data) at the given `col_idx, row_idx`
    /// offsets.
    pub fn get(&self, col_idx: usize, row_idx: usize) -> Data {
        match self.data.get(col_idx) {
            Some(Column::Int(col)) => match col.get(row_idx).unwrap() {
                Some(data) => Data::Int(*data),
                None => Data::Null,
            },
            Some(Column::Bool(col)) => match col.get(row_idx).unwrap() {
                Some(data) => Data::Bool(*data),
                None => Data::Null,
            },
            Some(Column::Float(col)) => match col.get(row_idx).unwrap() {
                Some(data) => Data::Float(*data),
                None => Data::Null,
            },
            Some(Column::String(col)) => match col.get(row_idx).unwrap() {
                Some(data) => Data::String(data.clone()),
                None => Data::Null,
            },
            None => panic!("Column index out of bounds"),
        }
    }

    /// Get the index of the `Column` with the given `col_name`. Returns `Some`
    /// if a `Column` with the given name exists, or `None` otherwise.
    pub fn get_col(&self, col_name: &str) -> Option<usize> {
        self.schema.col_idx(col_name)
    }

    /// Get the index of the `Row` with the given `row_name`. Returns `Some`
    /// if a `Row` with the given name exists, or `None` otherwise.
    pub fn get_row(&self, row_name: &str) -> Option<usize> {
        self.schema.row_idx(row_name)
    }

    /// Mutates the value in this `DataFrame` at the given `col_idx, row_idx`
    /// to be changed to the given `data`.
    pub fn set_int(
        &mut self,
        col_idx: usize,
        row_idx: usize,
        data: i64,
    ) -> Result<(), DFError> {
        if let Some(DataType::Int) = self.schema.schema.get(col_idx) {
            match self.data.get_mut(col_idx) {
                Some(Column::Int(col)) => {
                    *col.get_mut(row_idx).unwrap_or_else(|| {
                        panic!("Err: row idx out of bounds")
                    }) = Some(data)
                }
                _ => unreachable!("Something is horribly wrong"),
            }
        } else {
            panic!("Err: col idx out of bounds or col is not of int type")
        }
    }

    /// Mutates the value in this `DataFrame` at the given `col_idx, row_idx`
    /// to be changed to the given `data`.
    pub fn set_float(&mut self, col_idx: usize, row_idx: usize, data: f64) {
        if let Some(DataType::Float) = self.schema.schema.get(col_idx) {
            match self.data.get_mut(col_idx) {
                Some(Column::Float(col)) => {
                    *col.get_mut(row_idx).unwrap_or_else(|| {
                        panic!("Err: row idx out of bounds")
                    }) = Some(data)
                }
                _ => unreachable!("Something is horribly wrong"),
            }
        } else {
            panic!("Err: col idx out of bounds or col is not of float type")
        }
    }

    /// Mutates the value in this `DataFrame` at the given `col_idx, row_idx`
    /// to be changed to the given `data`.
    pub fn set_bool(&mut self, col_idx: usize, row_idx: usize, data: bool) {
        if let Some(DataType::Bool) = self.schema.schema.get(col_idx) {
            match self.data.get_mut(col_idx) {
                Some(Column::Bool(col)) => {
                    *col.get_mut(row_idx).unwrap_or_else(|| {
                        panic!("Err: row idx out of bounds")
                    }) = Some(data)
                }
                _ => unreachable!("Something is horribly wrong"),
            }
        } else {
            panic!("Err: col idx out of bounds or col is not of bool type")
        }
    }

    /// Mutates the value in this `DataFrame` at the given `col_idx, row_idx`
    /// to be changed to the given `data`.
    pub fn set_string(&mut self, col_idx: usize, row_idx: usize, data: String) {
        if let Some(DataType::String) = self.schema.schema.get(col_idx) {
            match self.data.get_mut(col_idx) {
                Some(Column::String(col)) => {
                    *col.get_mut(row_idx).unwrap_or_else(|| {
                        panic!("Err: row idx out of bounds")
                    }) = Some(data)
                }
                _ => unreachable!("Something is horribly wrong"),
            }
        } else {
            panic!("Err: col idx out of bounds or col is not of string type")
        }
    }

    /// Set the fields of the given `Row` struct with values from the row at
    /// the given `idx`.  If the row is not form the same schema as this
    /// `DataFrame`, results are undefined.
    pub fn fill_row(&self, idx: usize, row: &mut Row) {
        for (c_idx, col) in self.data.iter().enumerate() {
            match col {
                Column::Int(c) => match c.get(idx).unwrap() {
                    Some(x) => row.set_int(c_idx, *x),
                    None => row.set_null(c_idx),
                },
                Column::Float(c) => match c.get(idx).unwrap() {
                    Some(x) => row.set_float(c_idx, *x),
                    None => row.set_null(c_idx),
                },
                Column::Bool(c) => match c.get(idx).unwrap() {
                    Some(x) => row.set_bool(c_idx, *x),
                    None => row.set_null(c_idx),
                },
                Column::String(c) => match c.get(idx).unwrap() {
                    Some(x) => row.set_string(c_idx, x.clone()),
                    None => row.set_null(c_idx),
                },
            }
        }
    }

    /// Add a `Row` at the end of this `DataFrame`. Panics if the row has
    /// a `Schema` different than the `Schema` for this `DataFrame`.
    pub fn add_row(&mut self, row: &Row) {
        if row.schema != self.schema.schema {
            panic!("Err incompatible row")
        }
        for (data, column) in row.data.iter().zip(self.data.iter_mut()) {
            match (data, column) {
                (Data::Int(n), Column::Int(l)) => l.push(Some(*n)),
                (Data::Float(n), Column::Float(l)) => l.push(Some(*n)),
                (Data::Bool(n), Column::Bool(l)) => l.push(Some(*n)),
                (Data::String(n), Column::String(l)) => l.push(Some(n.clone())),
                (Data::Null, Column::Int(l)) => l.push(None),
                (Data::Null, Column::Float(l)) => l.push(None),
                (Data::Null, Column::Bool(l)) => l.push(None),
                (Data::Null, Column::String(l)) => l.push(None),
                (_, _) => panic!("Err: incampatible row"),
            }
        }
    }

    pub fn map<T: Rower>(&self, rower: &mut T) {
        map_helper(self, rower, 0, self.nrows());
    }

    // NOTE: crossbeam might remove the 'static
    /*pub fn pmap<T: Rower + Clone + Send>(&'static self, rower: &'static mut T) {
        //let mut rowers = Vec::new();
        let mut threads = Vec::new();
        //for _ in 0..self.n_threads - 1 {
        //    rowers.push(&mut rower.clone());
        //}
        //rowers.insert(0, rower);

        let rowers = vec![*rower; self.n_threads];
        let step = self.nrows() / self.n_threads; // +1 for this thread
        let mut from = 0;
        for i in 0..self.n_threads - 1 {
            threads.push(thread::spawn(move || {
                map_helper::<T>(&self, rowers.get_mut(i).unwrap(), from, from + step)
            }));
            from += step;
        }

        map_helper::<T>(
            self,
            rowers.get_mut(self.n_threads).unwrap(),
            from,
            self.nrows(),
        );

        for thread in threads {
            thread.join().unwrap();
        }

        //for (i, r) in rowers.iter_mut().enumerate().rev().skip(1) {
        //    r.join(rowers.get_mut(i + 1).unwrap());
        //}
    }*/

    /// Return the number of rows in this `DataFrame`.
    pub fn n_rows(&self) -> usize {
        self.schema.length()
    }

    /// Return the number of columns in this `DataFrame`.
    pub fn n_cols(&self) -> usize {
        self.schema.width()
    }
}

fn map_helper<T: Rower>(
    df: &DataFrame,
    rower: &mut T,
    start: usize,
    end: usize,
) {
    let mut row = Row::new(&df.schema);
    // NOTE: IS THIS THE ~10% slower way to do counted loop???? @tom
    for i in start..end {
        df.fill_row(i, &mut row);
        rower.visit(&mut row);
    }
}
