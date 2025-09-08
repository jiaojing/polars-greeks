# polars-greeks

High-performance Black-Scholes options pricing and Greeks calculation plugin for Polars.

Built with Rust for maximum performance, providing both DataFrame batch processing and scalar calculation interfaces.

## Features

- 🚀 **High Performance**: Rust implementation with zero-copy data processing
- 📊 **Dual API**: DataFrame batch processing + scalar direct calculation  
- 🎯 **Complete Greeks**: Delta, Gamma, Theta, Vega, Rho, Vanna, Volga, Charm, Speed, Zomma
- 💾 **Memory Efficient**: Pre-allocated vectors, avoid unnecessary computations
- ✅ **Well Tested**: Comprehensive test suite with mathematical accuracy validation

## Quick Start

### Installation

```bash
pip install polars-greeks
```

### DataFrame Batch Processing

```python
import polars as pl
import polars_greeks as greeks

df = pl.DataFrame({
    "spot": [100.0, 105.0, 95.0],
    "strike": [100.0, 100.0, 100.0],
    "time_to_expiry": [0.25, 0.25, 0.25], 
    "volatility": [0.2, 0.2, 0.2],
    "is_call": [True, False, True]
})

result = df.select(
    greeks.df(
        pl.col("spot"),
        strike=pl.col("strike"),
        time_to_expiry=pl.col("time_to_expiry"),
        volatility=pl.col("volatility"),
        r=pl.lit(0.05),
        q=pl.lit(0.0),
        is_call=pl.col("is_call"),
        greeks=["delta", "vega", "gamma", "theta"]
    ).alias("greeks_result")
).select(
    pl.col("greeks_result").struct.field("delta").alias("delta"),
    pl.col("greeks_result").struct.field("vega").alias("vega"),
    pl.col("greeks_result").struct.field("gamma").alias("gamma"),
    pl.col("greeks_result").struct.field("theta").alias("theta")
)
```

### Scalar Direct Calculation

```python
import polars_greeks as greeks

result = greeks.scalar(
    spot=100.0,
    strike=100.0,
    time_to_expiry=0.25,
    volatility=0.2,
    r=0.05,
    q=0.0,
    is_call=True,
    greeks=["delta", "vega", "gamma"]
)

print(result)
# {'delta': 0.56, 'vega': 19.8, 'gamma': 0.019}
```

## Supported Greeks

- **delta**: Price sensitivity to underlying asset price changes
- **gamma**: Delta sensitivity to underlying asset price changes  
- **theta**: Price sensitivity to time decay
- **vega**: Price sensitivity to volatility changes
- **rho**: Price sensitivity to interest rate changes
- **vanna**: Delta sensitivity to volatility changes
- **volga**: Vega sensitivity to volatility changes  
- **charm**: Delta sensitivity to time decay
- **speed**: Gamma sensitivity to underlying asset price changes
- **zomma**: Gamma sensitivity to volatility changes

## Performance

- **Zero-copy processing**: Direct operations on Polars Series
- **Batch optimization**: Single pass calculation of multiple Greeks
- **Selective computation**: Only calculate requested Greeks
- **Memory efficient**: Pre-allocated vectors, no unnecessary allocations

Typical performance: **100K+ calculations per second** for multiple Greeks.

## Requirements

- Python ≥ 3.8
- Polars ≥ 0.20.0

## License

MIT