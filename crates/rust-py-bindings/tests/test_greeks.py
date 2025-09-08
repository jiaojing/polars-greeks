"""Test Greeks calculations"""

import polars as pl
import polars_greeks as greeks
import pytest
from math import isclose


def test_basic_greeks_calculation():
    """Test basic Black-Scholes Greeks calculation"""
    df = pl.DataFrame({
        'spot': [100.0],
        'strike': [100.0],
        'time_to_expiry': [0.25],
        'volatility': [0.2],
        'is_call': [True]
    })
    
    result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.col('is_call'),
            greeks=['vega', 'delta', 'gamma']
        ).alias('greeks_result')
    ).select(
        pl.col('greeks_result').struct.field('delta').alias('delta'),
        pl.col('greeks_result').struct.field('vega').alias('vega'),
        pl.col('greeks_result').struct.field('gamma').alias('gamma')
    )
    
    # Extract values
    delta = result['delta'][0]
    vega = result['vega'][0] 
    gamma = result['gamma'][0]
    
    # Basic sanity checks
    assert 0.4 < delta < 0.7, f"Delta {delta} should be between 0.4 and 0.7 for ATM call"
    assert vega > 0, f"Vega {vega} should be positive"
    assert gamma > 0, f"Gamma {gamma} should be positive"


def test_put_call_parity():
    """Test that put and call delta sum to 1 (approximately)"""
    df = pl.DataFrame({
        'spot': [100.0],
        'strike': [100.0],
        'time_to_expiry': [0.25],
        'volatility': [0.2],
    })
    
    # Call delta
    call_result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.lit(True),
            greeks=['delta']
        ).alias('call_greeks')
    ).select(
        pl.col('call_greeks').struct.field('delta').alias('call_delta')
    )
    
    # Put delta  
    put_result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.lit(False),
            greeks=['delta']
        ).alias('put_greeks')
    ).select(
        pl.col('put_greeks').struct.field('delta').alias('put_delta')
    )
    
    call_delta = call_result['call_delta'][0]
    put_delta = put_result['put_delta'][0]
    
    # Put-call parity: call_delta - put_delta = 1 (when r=0, q=0)
    # With r>0, call_delta - put_delta = exp(-q*T) ≈ 1
    delta_diff = call_delta - put_delta
    assert isclose(delta_diff, 1.0, abs_tol=0.1), f"Call-Put delta difference {delta_diff} should be close to 1"


def test_multiple_strikes():
    """Test Greeks calculation across multiple strikes"""
    df = pl.DataFrame({
        'spot': [100.0, 100.0, 100.0],
        'strike': [90.0, 100.0, 110.0],  # ITM, ATM, OTM
        'time_to_expiry': [0.25, 0.25, 0.25],
        'volatility': [0.2, 0.2, 0.2],
        'is_call': [True, True, True]
    })
    
    result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.col('is_call'),
            greeks=['delta']
        ).alias('greeks_result')
    ).select(
        pl.col('greeks_result').struct.field('delta').alias('delta')
    )
    
    deltas = result['delta'].to_list()
    
    # ITM call should have higher delta than OTM call
    assert deltas[0] > deltas[1] > deltas[2], f"Delta should decrease from ITM to OTM: {deltas}"


if __name__ == "__main__":
    test_basic_greeks_calculation()
    test_put_call_parity()
    test_multiple_strikes()
    print("All tests passed!")
