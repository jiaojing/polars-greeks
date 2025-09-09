use polars::prelude::*;
use pyo3_polars::derive::polars_expr;
use rust_core::black_scholes::{BlackScholesModel, OptionType};
use serde::Deserialize;
use std::collections::HashSet;

struct GreeksVec {
    len: usize,
    delta: Option<Vec<f64>>,
    gamma: Option<Vec<f64>>,
    theta: Option<Vec<f64>>,
    vega: Option<Vec<f64>>,
    rho: Option<Vec<f64>>,
    vanna: Option<Vec<f64>>,
    volga: Option<Vec<f64>>,
    charm: Option<Vec<f64>>,
    speed: Option<Vec<f64>>,
    zomma: Option<Vec<f64>>,
}

impl GreeksVec {
    fn from_kwargs(len: usize, kwargs: &GreeksKwargs) -> Self {
        let maybe_vec = |name: &str| kwargs.hash_greeks(name).then(|| Vec::with_capacity(len));

        Self {
            len,
            delta: maybe_vec("delta"),
            gamma: maybe_vec("gamma"),
            theta: maybe_vec("theta"),
            vega: maybe_vec("vega"),
            rho: maybe_vec("rho"),
            vanna: maybe_vec("vanna"),
            volga: maybe_vec("volga"),
            charm: maybe_vec("charm"),
            speed: maybe_vec("speed"),
            zomma: maybe_vec("zomma"),
        }
    }

    fn collect_by(&mut self, bs: &BlackScholesModel, option_type: OptionType) {
        self.delta
            .as_mut()
            .map(|vec| vec.push(bs.delta(option_type)));
        self.gamma.as_mut().map(|vec| vec.push(bs.gamma()));
        self.theta
            .as_mut()
            .map(|vec| vec.push(bs.theta(option_type)));
        self.vega.as_mut().map(|vec| vec.push(bs.vega()));
        self.rho.as_mut().map(|vec| vec.push(bs.rho(option_type)));
        self.vanna.as_mut().map(|vec| vec.push(bs.vanna()));
        self.volga.as_mut().map(|vec| vec.push(bs.volga()));
        self.charm
            .as_mut()
            .map(|vec| vec.push(bs.charm(option_type)));
        self.speed.as_mut().map(|vec| vec.push(bs.speed()));
        self.zomma.as_mut().map(|vec| vec.push(bs.zomma()));
    }

    fn collect_by_default(&mut self) {
        // 推送NaN保持长度一致
        self.delta.as_mut().map(|vec| vec.push(f64::NAN));
        self.gamma.as_mut().map(|vec| vec.push(f64::NAN));
        self.theta.as_mut().map(|vec| vec.push(f64::NAN));
        self.vega.as_mut().map(|vec| vec.push(f64::NAN));
        self.rho.as_mut().map(|vec| vec.push(f64::NAN));
        self.vanna.as_mut().map(|vec| vec.push(f64::NAN));
        self.volga.as_mut().map(|vec| vec.push(f64::NAN));
        self.charm.as_mut().map(|vec| vec.push(f64::NAN));
        self.speed.as_mut().map(|vec| vec.push(f64::NAN));
        self.zomma.as_mut().map(|vec| vec.push(f64::NAN));
    }

    fn to_struct_series(self) -> PolarsResult<Series> {
        let mut series_vec = Vec::new();

        self.delta.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("delta".into(), vec).into_series());
        });
        self.gamma.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("gamma".into(), vec).into_series());
        });
        self.theta.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("theta".into(), vec).into_series());
        });
        self.vega.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("vega".into(), vec).into_series());
        });
        self.rho.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("rho".into(), vec).into_series());
        });
        self.vanna.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("vanna".into(), vec).into_series());
        });
        self.volga.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("volga".into(), vec).into_series());
        });
        self.charm.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("charm".into(), vec).into_series());
        });
        self.speed.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("speed".into(), vec).into_series());
        });
        self.zomma.map(|vec| {
            series_vec.push(Float64Chunked::from_vec("zomma".into(), vec).into_series());
        });

        let struct_chunked =
            StructChunked::from_series("greeks".into(), self.len, series_vec.iter())?;
        Ok(struct_chunked.into_series())
    }
}

#[derive(Deserialize)]
pub struct GreeksKwargs {
    #[serde(default = "default_greeks")]
    pub greeks: HashSet<String>,
}

impl GreeksKwargs {
    fn hash_greeks(&self, greek: impl Into<String>) -> bool {
        self.greeks.contains(&greek.into())
    }
}

fn default_greeks() -> HashSet<String> {
    HashSet::from(["vega".to_string(), "charm".to_string()])
}

fn infer_greeks_struct_schema(input_fields: &[Field], kwargs: GreeksKwargs) -> PolarsResult<Field> {
    // 校验参数数量
    if input_fields.len() != 7 {
        polars_bail!(InvalidOperation: "Expected 7 input fields, got {}", input_fields.len());
    }

    // 校验前6个字段必须是Float64
    for (i, field) in input_fields.iter().take(6).enumerate() {
        if !matches!(field.dtype(), DataType::Float64) {
            polars_bail!(InvalidOperation: "Field {} '{}' must be Float64, got {:?}", i, field.name(), field.dtype());
        }
    }

    // 校验第7个字段必须是Boolean
    if !matches!(input_fields[6].dtype(), DataType::Boolean) {
        polars_bail!(InvalidOperation: "Field 6 '{}' must be Boolean, got {:?}", input_fields[6].name(), input_fields[6].dtype());
    }

    let valid_greeks = [
        "delta", "gamma", "theta", "vega", "rho", "vanna", "volga", "charm", "speed", "zomma",
    ];

    // 校验Greek名称
    for greek in &kwargs.greeks {
        if !valid_greeks.contains(&greek.as_str()) {
            polars_bail!(InvalidOperation: "Invalid Greek: '{}'", greek);
        }
    }

    let fields: Vec<Field> = valid_greeks
        .into_iter()
        .filter(|&name| kwargs.hash_greeks(name))
        .map(|name| Field::new(name.into(), DataType::Float64))
        .collect();

    Ok(Field::new("greeks".into(), DataType::Struct(fields)))
}

#[polars_expr(output_type_func_with_kwargs=infer_greeks_struct_schema)]
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
    let mut greeks_vec = GreeksVec::from_kwargs(len, &kwargs);

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

                if let Ok(bs) = BlackScholesModel::new(s, k, t, vol, r, q) {
                    greeks_vec.collect_by(&bs, option_type);
                } else {
                    // 推送默认值或NaN，保持长度一致
                    greeks_vec.collect_by_default();
                }
            },
        );

    greeks_vec.to_struct_series()
}
