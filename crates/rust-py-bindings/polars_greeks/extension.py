"""
Polars Greeks namespace extension
"""

import polars as pl
from . import greeks as greeks_module


@pl.api.register_expr_namespace("greeks")
class GreeksNamespace:
    """Greeks calculations namespace for Polars expressions"""
    
    def __init__(self, expr: pl.Expr):
        self._expr = expr
    
    
    def calc_basic(
        self,
        strike,
        time_to_expiry,
        volatility,
        r,
        q, 
        is_call,
        greeks: list[str] = None,
    ) -> pl.Expr:
        """
        Calculate basic Black-Scholes Greeks (delta, gamma, theta, vega, rho)
        """
        return greeks_module.calc_basic(
            self._expr, strike, time_to_expiry, volatility, r, q, is_call, greeks
        )
