"""
Greeks calculations using register_plugin_function
"""

import polars as pl
from polars.plugins import register_plugin_function
from pathlib import Path
from typing import Union, Dict, List, Optional

# Plugin path - find the correct shared library dynamically
PLUGIN_PATH = Path(__file__).parent

# Import the Rust function
from ._internal import calc as _calc_rust


def df(
    spot_expr: Union[str, pl.Expr],
    strike: Union[str, pl.Expr, float],
    time_to_expiry: Union[str, pl.Expr, float],
    volatility: Union[str, pl.Expr, float],
    r: Union[str, pl.Expr, float] = 0.0,
    q: Union[str, pl.Expr, float] = 0.0,
    is_call: Union[str, pl.Expr, bool] = True,
    greeks: list[str] = None,
) -> pl.Expr:
    """Calculate Black-Scholes Greeks for DataFrame (batch processing)"""
    if greeks is None:
        greeks = ["vega"]  # Default to vega only
    
    return register_plugin_function(
        plugin_path=PLUGIN_PATH,
        function_name="calc_basic",
        args=[spot_expr, strike, time_to_expiry, volatility, r, q, is_call],
        kwargs={"greeks": greeks},
        is_elementwise=False,
    )


def scalar(
    spot: float,
    strike: float, 
    time_to_expiry: float,
    volatility: float,
    r: float = 0.0,
    q: float = 0.0,
    is_call: bool = True,
    greeks: Optional[List[str]] = None,
) -> Dict[str, float]:
    """
    Calculate Black-Scholes Greeks for a single option (scalar values)
    
    Args:
        spot: Current price of the underlying asset
        strike: Strike price of the option
        time_to_expiry: Time to expiry in years
        volatility: Volatility of the underlying asset
        r: Risk-free interest rate (default: 0.0)
        q: Dividend yield (default: 0.0) 
        is_call: True for call option, False for put option (default: True)
        greeks: List of Greeks to calculate (default: ["delta", "gamma", "theta", "vega", "rho"])
        
    Returns:
        Dictionary mapping Greek names to their values
        
    Example:
        >>> import polars_greeks as greeks
        >>> result = greeks.scalar(
        ...     spot=100.0,
        ...     strike=100.0,
        ...     time_to_expiry=0.25,
        ...     volatility=0.2,
        ...     greeks=["delta", "vega", "gamma"]
        ... )
        >>> print(result["delta"])
        0.56
    """
    if greeks is None:
        greeks = ["delta", "gamma", "theta", "vega", "rho"]
    
    return _calc_rust(spot, strike, time_to_expiry, volatility, r, q, is_call, greeks)
