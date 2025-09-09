mod polars_greeks;

use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3_polars::PolarsAllocator;
use std::collections::HashMap;
use rust_core::black_scholes::{BlackScholesModel, OptionType};

#[global_allocator]
static ALLOC: PolarsAllocator = PolarsAllocator::new();

pub trait Apply: Sized {
    fn apply<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        f(&mut self);
        self
    }
}

impl<T> Apply for T {}


#[pyfunction]
fn calc(
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    volatility: f64,
    r: f64,
    q: f64,
    is_call: bool,
    greeks: Vec<String>,
) -> PyResult<HashMap<String, f64>> {
    let model = BlackScholesModel::new(spot, strike, time_to_expiry, volatility, r, q)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
    
    let option_type = if is_call { OptionType::Call } else { OptionType::Put };
    let mut result = HashMap::new();
    
    for greek in greeks {
        let value = match greek.as_str() {
            "delta" => model.delta(option_type),
            "gamma" => model.gamma(),
            "theta" => model.theta(option_type),
            "vega" => model.vega(),
            "rho" => model.rho(option_type),
            "vanna" => model.vanna(),
            "volga" => model.volga(),
            "charm" => model.charm(option_type),
            "speed" => model.speed(),
            "zomma" => model.zomma(),
            "price" => model.option_price(option_type),
            "intrinsic" => model.intrinsic_value(option_type),
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Unknown greek: {}", greek)
            )),
        };
        result.insert(greek, value);
    }
    
    Ok(result)
}

#[pymodule]
fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calc, m)?)?;
    Ok(())
}
