"""Performance tests for Greeks calculations"""

import polars as pl
import polars_greeks as greeks
import time
import numpy as np


def test_performance_large_dataset():
    """Test performance on larger dataset"""
    # Generate 10,000 option scenarios
    n = 10000
    np.random.seed(42)
    
    df = pl.DataFrame({
        'spot': np.random.uniform(80, 120, n),
        'strike': np.random.uniform(90, 110, n),
        'time_to_expiry': np.random.uniform(0.1, 1.0, n),
        'volatility': np.random.uniform(0.1, 0.5, n),
        'is_call': np.random.choice([True, False], n)
    })
    
    print(f"Testing performance on {n:,} option scenarios...")
    
    start_time = time.time()
    
    result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.col('is_call'),
            greeks=['vega', 'delta', 'gamma', 'theta']
        ).alias('greeks_result')
    )
    
    end_time = time.time()
    elapsed = end_time - start_time
    
    print(f"Calculated {n:,} Greeks in {elapsed:.3f} seconds")
    print(f"Rate: {n/elapsed:,.0f} calculations per second")
    
    # Should be quite fast
    assert elapsed < 1.0, f"Performance too slow: {elapsed:.3f}s for {n:,} calculations"
    
    # Verify we got results
    assert result.shape[0] == n
    
    return elapsed


def test_memory_usage():
    """Test memory efficiency"""
    # Test that we can handle reasonable dataset sizes
    n = 50000
    
    df = pl.DataFrame({
        'spot': [100.0] * n,
        'strike': [100.0] * n,
        'time_to_expiry': [0.25] * n,
        'volatility': [0.2] * n,
        'is_call': [True] * n
    })
    
    print(f"Testing memory usage on {n:,} identical scenarios...")
    
    result = df.select(
        greeks.df(
            pl.col('spot'),
            strike=pl.col('strike'),
            time_to_expiry=pl.col('time_to_expiry'),
            volatility=pl.col('volatility'),
            r=pl.lit(0.05),
            q=pl.lit(0.0),
            is_call=pl.col('is_call'),
            greeks=['delta', 'gamma', 'vega']
        ).alias('greeks_result')
    )
    
    # Should complete without memory errors
    assert result.shape[0] == n
    print(f"Successfully processed {n:,} scenarios")


if __name__ == "__main__":
    test_performance_large_dataset()
    test_memory_usage()
    print("Performance tests completed!")
