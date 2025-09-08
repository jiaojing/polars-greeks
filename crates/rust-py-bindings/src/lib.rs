mod polars_greeks;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3_polars::PolarsAllocator;

#[global_allocator]
static ALLOC: PolarsAllocator = PolarsAllocator::new();

#[pymodule]
fn _internal(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
