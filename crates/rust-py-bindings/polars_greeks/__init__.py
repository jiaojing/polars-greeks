"""
Polars Greeks Plugin - Black-Scholes Greeks calculations for Polars
"""

from .functions import calc_basic

__version__ = "0.1.0"

__all__ = [
    "__version__",
    "calc_basic",
]
