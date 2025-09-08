"""Test scalar calc interface"""

import polars_greeks as greeks
import pytest
from math import isclose


def test_calc_scalar_basic():
    """Test basic scalar Greeks calculation"""
    result = greeks.scalar(
        spot=100.0,
        strike=100.0,
        time_to_expiry=0.25,
        volatility=0.2,
        r=0.05,
        q=0.0,
        is_call=True,
        greeks=['delta', 'vega', 'gamma']
    )
    
    # Basic sanity checks
    assert isinstance(result, dict)
    assert 'delta' in result
    assert 'vega' in result
    assert 'gamma' in result
    
    delta = result['delta']
    vega = result['vega']
    gamma = result['gamma']
    
    assert 0.4 < delta < 0.7, f"Delta {delta} should be between 0.4 and 0.7 for ATM call"
    assert vega > 0, f"Vega {vega} should be positive"
    assert gamma > 0, f"Gamma {gamma} should be positive"


def test_calc_put_call_parity():
    """Test put-call parity for scalar interface"""
    params = {
        'spot': 100.0,
        'strike': 100.0,
        'time_to_expiry': 0.25,
        'volatility': 0.2,
        'r': 0.05,
        'q': 0.0,
        'greeks': ['delta']
    }
    
    call_result = greeks.scalar(**params, is_call=True)
    put_result = greeks.scalar(**params, is_call=False)
    
    call_delta = call_result['delta']
    put_delta = put_result['delta']
    
    # Put-call parity: call_delta - put_delta ≈ exp(-q*T) ≈ 1 (when q=0)
    delta_diff = call_delta - put_delta
    assert isclose(delta_diff, 1.0, abs_tol=0.1), f"Call-Put delta difference {delta_diff} should be close to 1"


def test_calc_all_greeks():
    """Test calculating all available Greeks"""
    result = greeks.scalar(
        spot=100.0,
        strike=100.0,
        time_to_expiry=0.25,
        volatility=0.2,
        r=0.05,
        q=0.0,
        is_call=True,
        greeks=['delta', 'gamma', 'theta', 'vega', 'rho', 'vanna', 'volga', 'charm', 'speed', 'zomma']
    )
    
    # Check all Greeks are present and have reasonable values
    assert len(result) == 10
    assert all(isinstance(v, float) for v in result.values())
    
    # Basic sanity checks for ATM call
    assert 0.4 < result['delta'] < 0.7
    assert result['gamma'] > 0
    assert result['theta'] < 0  # Time decay
    assert result['vega'] > 0


def test_calc_with_price():
    """Test calculating price alongside Greeks"""
    result = greeks.scalar(
        spot=100.0,
        strike=100.0,
        time_to_expiry=0.25,
        volatility=0.2,
        r=0.05,
        q=0.0,
        is_call=True,
        greeks=['price', 'intrinsic', 'delta']
    )
    
    assert 'price' in result
    assert 'intrinsic' in result  
    assert 'delta' in result
    
    # ATM call should have positive price
    assert result['price'] > 0
    # ATM call intrinsic should be positive (current spot discounted vs strike discounted)
    assert result['intrinsic'] > 0


def test_calc_default_greeks():
    """Test default Greeks selection"""
    result = greeks.scalar(
        spot=100.0,
        strike=100.0,
        time_to_expiry=0.25,
        volatility=0.2
    )
    
    # Should have default Greeks
    expected_greeks = {'delta', 'gamma', 'theta', 'vega', 'rho'}
    assert set(result.keys()) == expected_greeks


def test_calc_invalid_greek():
    """Test handling of invalid Greek name"""
    with pytest.raises(ValueError, match="Unknown greek"):
        greeks.scalar(
            spot=100.0,
            strike=100.0,
            time_to_expiry=0.25,
            volatility=0.2,
            greeks=['invalid_greek']
        )


def test_calc_invalid_parameters():
    """Test parameter validation"""
    # Negative spot price
    with pytest.raises(ValueError):
        greeks.scalar(
            spot=-100.0,
            strike=100.0,
            time_to_expiry=0.25,
            volatility=0.2
        )
    
    # Zero volatility
    with pytest.raises(ValueError):
        greeks.scalar(
            spot=100.0,
            strike=100.0,
            time_to_expiry=0.25,
            volatility=0.0
        )


def test_calc_consistency_with_dataframe():
    """Test that scalar calc gives same results as DataFrame calc_basic"""
    # Scalar calculation
    scalar_result = greeks.scalar(
        spot=100.0,
        strike=100.0,
        time_to_expiry=0.25,
        volatility=0.2,
        r=0.05,
        q=0.0,
        is_call=True,
        greeks=['delta', 'vega', 'gamma']
    )
    
    # DataFrame calculation
    import polars as pl
    df = pl.DataFrame({
        'spot': [100.0],
        'strike': [100.0],
        'time_to_expiry': [0.25],
        'volatility': [0.2],
        'is_call': [True]
    })
    
    df_result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.col('is_call'),
            greeks=['delta', 'vega', 'gamma']
        ).alias('greeks_result')
    ).select(
        pl.col('greeks_result').struct.field('delta').alias('delta'),
        pl.col('greeks_result').struct.field('vega').alias('vega'),
        pl.col('greeks_result').struct.field('gamma').alias('gamma')
    )
    
    # Compare results
    df_delta = df_result['delta'][0]
    df_vega = df_result['vega'][0]
    df_gamma = df_result['gamma'][0]
    
    assert isclose(scalar_result['delta'], df_delta, rel_tol=1e-10)
    assert isclose(scalar_result['vega'], df_vega, rel_tol=1e-10)
    assert isclose(scalar_result['gamma'], df_gamma, rel_tol=1e-10)


if __name__ == "__main__":
    test_calc_scalar_basic()
    test_calc_put_call_parity()
    test_calc_all_greeks()
    test_calc_with_price()
    test_calc_default_greeks()
    print("All scalar calc tests passed!")