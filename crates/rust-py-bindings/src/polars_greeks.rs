use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use rust_core::black_scholes::{BlackScholesModel, Greeks, OptionType};
use serde::Deserialize;
use std::collections::HashSet;


#[derive(Debug, Clone)]
struct GreeksFlags {
    delta: bool,
    gamma: bool,
    theta: bool,
    vega: bool,
    rho: bool,
    vanna: bool,
    volga: bool,
    charm: bool,
    speed: bool,
    zomma: bool,
}

impl GreeksFlags {
    fn from_kwargs(kwargs: &GreeksKwargs) -> Self {
        let greeks = &kwargs.greeks;
        Self {
            delta: greeks.contains(&"delta".to_string()),
            gamma: greeks.contains(&"gamma".to_string()),
            theta: greeks.contains(&"theta".to_string()),
            vega: greeks.contains(&"vega".to_string()),
            rho: greeks.contains(&"rho".to_string()),
            vanna: greeks.contains(&"vanna".to_string()),
            volga: greeks.contains(&"volga".to_string()),
            charm: greeks.contains(&"charm".to_string()),
            speed: greeks.contains(&"speed".to_string()),
            zomma: greeks.contains(&"zomma".to_string()),
        }
    }
}

fn greeks_from_bs(bs: &BlackScholesModel, option_type: OptionType, flags: &GreeksFlags) -> Greeks {
    Greeks {
        delta: if flags.delta {
            bs.delta(option_type)
        } else {
            0.0
        },
        gamma: if flags.gamma { bs.gamma() } else { 0.0 },
        theta: if flags.theta {
            bs.theta(option_type)
        } else {
            0.0
        },
        vega: if flags.vega { bs.vega() } else { 0.0 },
        rho: if flags.rho { bs.rho(option_type) } else { 0.0 },
        vanna: if flags.vanna { bs.vanna() } else { 0.0 },
        volga: if flags.volga { bs.volga() } else { 0.0 },
        charm: if flags.charm {
            bs.charm(option_type)
        } else {
            0.0
        },
        speed: if flags.speed { bs.speed() } else { 0.0 },
        zomma: if flags.zomma { bs.zomma() } else { 0.0 },
    }
}

struct GreeksVec {
    delta: Vec<f64>,
    gamma: Vec<f64>,
    theta: Vec<f64>,
    vega: Vec<f64>,
    rho: Vec<f64>,
    vanna: Vec<f64>,
    volga: Vec<f64>,
    charm: Vec<f64>,
    speed: Vec<f64>,
    zomma: Vec<f64>,
}

impl GreeksVec {
    fn with_capacity(len: usize) -> Self {
        Self {
            delta: Vec::with_capacity(len),
            gamma: Vec::with_capacity(len),
            theta: Vec::with_capacity(len),
            vega: Vec::with_capacity(len),
            rho: Vec::with_capacity(len),
            vanna: Vec::with_capacity(len),
            volga: Vec::with_capacity(len),
            charm: Vec::with_capacity(len),
            speed: Vec::with_capacity(len),
            zomma: Vec::with_capacity(len),
        }
    }

    fn push(&mut self, greeks: Greeks) {
        self.delta.push(greeks.delta);
        self.gamma.push(greeks.gamma);
        self.theta.push(greeks.theta);
        self.vega.push(greeks.vega);
        self.rho.push(greeks.rho);
        self.vanna.push(greeks.vanna);
        self.volga.push(greeks.volga);
        self.charm.push(greeks.charm);
        self.speed.push(greeks.speed);
        self.zomma.push(greeks.zomma);
    }

    fn to_struct_series(self) -> PolarsResult<Series> {
        let row_count = self.delta.len();

        // 直接从Vec<f64>构造Series - 零拷贝move操作
        let series_vec = vec![
            Float64Chunked::from_vec("delta".into(), self.delta).into_series(),
            Float64Chunked::from_vec("gamma".into(), self.gamma).into_series(),
            Float64Chunked::from_vec("theta".into(), self.theta).into_series(),
            Float64Chunked::from_vec("vega".into(), self.vega).into_series(),
            Float64Chunked::from_vec("rho".into(), self.rho).into_series(),
            Float64Chunked::from_vec("vanna".into(), self.vanna).into_series(),
            Float64Chunked::from_vec("volga".into(), self.volga).into_series(),
            Float64Chunked::from_vec("charm".into(), self.charm).into_series(),
            Float64Chunked::from_vec("speed".into(), self.speed).into_series(),
            Float64Chunked::from_vec("zomma".into(), self.zomma).into_series(),
        ];

        // ✅ 使用StructChunked::from_series正确构造Struct
        let struct_chunked =
            StructChunked::from_series("all_greeks".into(), row_count, series_vec.iter())?;

        Ok(struct_chunked.into_series())
    }
}

#[derive(Deserialize)]
pub struct GreeksKwargs {
    #[serde(default = "default_greeks")]
    pub greeks: HashSet<String>,
}

fn default_greeks() -> HashSet<String> {
    HashSet::from(["vega".to_string(), "charm".to_string()])
}

fn infer_greeks_struct_schema(_input_fields: &[Field]) -> PolarsResult<Field> {
    // 返回完整的Greeks结构体 - 包含所有可能的指标
    let fields = vec![
        Field::new("delta".into(), DataType::Float64),
        Field::new("gamma".into(), DataType::Float64),
        Field::new("theta".into(), DataType::Float64),
        Field::new("vega".into(), DataType::Float64),
        Field::new("rho".into(), DataType::Float64),
        Field::new("vanna".into(), DataType::Float64),
        Field::new("volga".into(), DataType::Float64),
        Field::new("charm".into(), DataType::Float64),
        Field::new("speed".into(), DataType::Float64),
        Field::new("zomma".into(), DataType::Float64),
    ];
    Ok(Field::new("all_greeks".into(), DataType::Struct(fields)))
}

#[polars_expr(output_type_func=infer_greeks_struct_schema)]
pub fn calc_basic(inputs: &[Series], kwargs: GreeksKwargs) -> PolarsResult<Series> {
    let (s_series, k_series, t_series, vol_series, r_series_raw, q_series_raw, is_call_series) = (
        inputs[0].f64()?,
        inputs[1].f64()?,
        inputs[2].f64()?,
        inputs[3].f64()?,
        inputs[4].f64()?,
        inputs[5].f64()?,
        inputs[6].bool()?,
    );

    let len = s_series.len();

    // Handle pl.lit() expressions: broadcast length-1 series to match data length
    let r_series = if r_series_raw.len() == 1 && len > 1 {
        r_series_raw.new_from_index(0, len)
    } else {
        r_series_raw.clone()
    };
    let q_series = if q_series_raw.len() == 1 && len > 1 {
        q_series_raw.new_from_index(0, len)
    } else {
        q_series_raw.clone()
    };
    let mut greeks_vec = GreeksVec::with_capacity(len);

    // Determine which Greeks to calculate
    let flags = GreeksFlags::from_kwargs(&kwargs);

    // Single pass through all rows - calculate only requested Greeks
    s_series
        .into_iter()
        .zip(k_series.into_iter())
        .zip(t_series.into_iter())
        .zip(vol_series.into_iter())
        .zip(r_series.into_iter())
        .zip(q_series.into_iter())
        .zip(is_call_series.into_iter())
        .for_each(
            |((((((s_opt, k_opt), t_opt), vol_opt), r_opt), q_opt), is_call_opt)| {
                let (s, k, t, vol, r, q, is_call) = (
                    s_opt.unwrap_or(0.0),
                    k_opt.unwrap_or(0.0),
                    t_opt.unwrap_or(0.0),
                    vol_opt.unwrap_or(0.0),
                    r_opt.unwrap_or(0.0),
                    q_opt.unwrap_or(0.0),
                    is_call_opt.unwrap_or(true),
                );

                let option_type = if is_call {
                    OptionType::Call
                } else {
                    OptionType::Put
                };

                let greeks = BlackScholesModel::new(s, k, t, vol, r, q)
                    .map(|bs| greeks_from_bs(&bs, option_type, &flags))
                    .unwrap_or_default();

                greeks_vec.push(greeks);
            },
        );

    greeks_vec.to_struct_series()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_core::black_scholes::{BlackScholesModel, OptionType};

    #[test]
    fn test_black_scholes_calculation() {
        // Test the core Black-Scholes calculation directly
        let bs =
            BlackScholesModel::new(100.0, 100.0, 0.25, 0.20, 0.0, 0.0).expect("Valid parameters");

        let vega = bs.vega();
        let delta = bs.delta(OptionType::Call);
        let gamma = bs.gamma();
        let theta = bs.theta(OptionType::Call);

        // Basic sanity checks
        assert!(vega > 0.0, "Vega should be positive");
        assert!(
            delta > 0.0 && delta < 1.0,
            "Call delta should be between 0 and 1"
        );
        assert!(gamma > 0.0, "Gamma should be positive");
        assert!(theta < 0.0, "Call theta should be negative (time decay)");
    }

    #[test]
    fn test_struct_construction() {
        // Test that we can construct the expected struct correctly
        let delta_values = vec![0.5, 0.6];
        let vega_values = vec![0.2, 0.25];

        let delta_series = Float64Chunked::from_vec("delta".into(), delta_values).into_series();
        let vega_series = Float64Chunked::from_vec("vega".into(), vega_values).into_series();

        let series_refs = [&delta_series, &vega_series];
        let struct_result =
            StructChunked::from_series("greeks".into(), 2, series_refs.iter().copied());

        assert!(struct_result.is_ok(), "Should be able to create struct");
        let struct_chunked = struct_result.unwrap();
        assert_eq!(struct_chunked.len(), 2);
    }
}
