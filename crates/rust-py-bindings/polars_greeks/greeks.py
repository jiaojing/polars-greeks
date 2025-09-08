"""
Greeks calculations using register_plugin_function
"""

import polars as pl
from polars.plugins import register_plugin_function
from pathlib import Path
from polars.type_aliases import IntoExpr

# Plugin path - find the correct shared library dynamically
PLUGIN_PATH = Path(__file__).parent


def calc_basic(
    spot_expr,
    strike: IntoExpr,
    time_to_expiry: IntoExpr,
    volatility: IntoExpr,
    r: IntoExpr = 0.0,
    q: IntoExpr = 0.0,
    is_call: IntoExpr = True,
    greeks: list[str] = None,
) -> pl.Expr:
    """Calculate basic Black-Scholes Greeks (delta, gamma, theta, vega, rho)"""
    if greeks is None:
        greeks = ["vega"]  # Default to vega only
    
    return register_plugin_function(
        plugin_path=PLUGIN_PATH,
        function_name="calc_basic",
        args=[spot_expr, strike, time_to_expiry, volatility, r, q, is_call],
        kwargs={"greeks": greeks},
        is_elementwise=False,
    )
