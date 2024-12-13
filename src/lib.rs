use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyclass]
struct Counter {
    count: i32,
}

#[pymethods]
impl Counter {
    #[new]
    fn new() -> Self {
        Counter { count: 0 }
    }

    fn increment(&mut self) -> PyResult<()> {
        self.count += 1;
        Ok(())
    }

    fn get_count(&self) -> PyResult<i32> {
        Ok(self.count)
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Counter({})", self.count))
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_reversi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Counter>()?;
    Ok(())
}
