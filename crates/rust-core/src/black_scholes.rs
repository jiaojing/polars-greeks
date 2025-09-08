use statrs::distribution::{ContinuousCDF, Normal};
use std::cell::OnceCell;
use std::sync::OnceLock;

const EPSILON: f64 = 1e-12;

// Global standard normal distribution - thread-safe lazy initialization
static STANDARD_NORMAL: OnceLock<Normal> = OnceLock::new();

fn standard_normal() -> &'static Normal {
    STANDARD_NORMAL.get_or_init(|| Normal::new(0.0, 1.0).unwrap())
}

#[derive(Debug, Clone, Copy)]
pub enum OptionType {
    Call,
    Put,
}

/// Black-Scholes model - Linus-style: simple struct, no fancy caching
#[derive(Debug, Clone)]
pub struct BlackScholesModel {
    pub s: f64,     // Spot price
    pub k: f64,     // Strike price
    pub t: f64,     // Time to expiry (years)
    pub sigma: f64, // Volatility
    pub r: f64,     // Risk-free rate
    pub q: f64,     // Dividend yield

    // Cached expensive calculations - OnceCell for proper lazy initialization
    d1: OnceCell<f64>,
    d2: OnceCell<f64>,
    nd1: OnceCell<f64>,
    nd2: OnceCell<f64>,
    phi_d1: OnceCell<f64>,
}

impl BlackScholesModel {
    pub fn new(s: f64, k: f64, t: f64, sigma: f64, r: f64, q: f64) -> Result<Self, &'static str> {
        if s <= 0.0 {
            return Err("Spot price must be positive");
        }
        if k <= 0.0 {
            return Err("Strike price must be positive");
        }
        if t <= EPSILON {
            return Err("Time to maturity must be positive");
        }
        if sigma <= 0.0 {
            return Err("Volatility must be positive");
        }

        Ok(BlackScholesModel {
            s,
            k,
            t,
            sigma,
            r,
            q,
            d1: OnceCell::new(),
            d2: OnceCell::new(),
            nd1: OnceCell::new(),
            nd2: OnceCell::new(),
            phi_d1: OnceCell::new(),
        })
    }

    // Basic calculations - always compute, they're cheap
    #[inline]
    pub fn sqrt_t(&self) -> f64 {
        self.t.sqrt()
    }

    #[inline]
    pub fn sigma_sqrt_t(&self) -> f64 {
        self.sigma * self.sqrt_t()
    }

    #[inline]
    pub fn log_sk(&self) -> f64 {
        (self.s / self.k).ln()
    }

    #[inline]
    pub fn exp_neg_rt(&self) -> f64 {
        (-self.r * self.t).exp()
    }

    #[inline]
    pub fn exp_neg_qt(&self) -> f64 {
        (-self.q * self.t).exp()
    }

    // d1 calculation with caching - OnceCell style
    pub fn d1(&self) -> f64 {
        *self.d1.get_or_init(|| {
            (self.log_sk() + (self.r - self.q + 0.5 * self.sigma * self.sigma) * self.t)
                / self.sigma_sqrt_t()
        })
    }

    // d2 calculation with caching - OnceCell style
    pub fn d2(&self) -> f64 {
        *self.d2.get_or_init(|| self.d1() - self.sigma_sqrt_t())
    }

    // N(d1) with caching - OnceCell style
    pub fn nd1(&self) -> f64 {
        *self.nd1.get_or_init(|| norm_cdf(self.d1()))
    }

    // N(d2) with caching - OnceCell style
    pub fn nd2(&self) -> f64 {
        *self.nd2.get_or_init(|| norm_cdf(self.d2()))
    }

    // φ(d1) with caching - OnceCell style
    pub fn phi_d1(&self) -> f64 {
        *self.phi_d1.get_or_init(|| norm_pdf(self.d1()))
    }

    // Derived properties
    #[inline]
    pub fn n_minus_d1(&self) -> f64 {
        1.0 - self.nd1()
    }

    #[inline]
    pub fn n_minus_d2(&self) -> f64 {
        1.0 - self.nd2()
    }
}

// Math functions using statrs - professional statistical functions
#[inline]
pub fn norm_pdf(x: f64) -> f64 {
    use statrs::distribution::Continuous;
    standard_normal().pdf(x)
}

#[inline]
pub fn norm_cdf(x: f64) -> f64 {
    standard_normal().cdf(x)
}

#[inline]
fn to_valid_or_zero(x: f64) -> f64 {
    if x.is_finite() {
        x
    } else {
        0.0
    }
}

// Pricing functions
impl BlackScholesModel {
    /// Calculate option price
    pub fn option_price(&self, option_type: OptionType) -> f64 {
        let price = match option_type {
            OptionType::Call => {
                self.s * self.exp_neg_qt() * self.nd1() - self.k * self.exp_neg_rt() * self.nd2()
            }
            OptionType::Put => {
                self.k * self.exp_neg_rt() * self.n_minus_d2()
                    - self.s * self.exp_neg_qt() * self.n_minus_d1()
            }
        };
        to_valid_or_zero(price)
    }

    /// Calculate intrinsic value
    pub fn intrinsic_value(&self, option_type: OptionType) -> f64 {
        match option_type {
            OptionType::Call => {
                f64::max(0.0, self.s * self.exp_neg_qt() - self.k * self.exp_neg_rt())
            }
            OptionType::Put => {
                f64::max(0.0, self.k * self.exp_neg_rt() - self.s * self.exp_neg_qt())
            }
        }
    }

    /// Calculate delta
    pub fn delta(&self, option_type: OptionType) -> f64 {
        let delta = match option_type {
            OptionType::Call => self.exp_neg_qt() * self.nd1(),
            OptionType::Put => self.exp_neg_qt() * (self.nd1() - 1.0),
        };
        to_valid_or_zero(delta)
    }

    /// Calculate gamma (option type independent)
    pub fn gamma(&self) -> f64 {
        let gamma = self.phi_d1() * self.exp_neg_qt() / (self.s * self.sigma_sqrt_t());
        to_valid_or_zero(gamma)
    }

    /// Calculate theta
    pub fn theta(&self, option_type: OptionType) -> f64 {
        let t1 = -self.s * self.phi_d1() * self.sigma * self.exp_neg_qt() / (2.0 * self.sqrt_t());
        let t2 = match option_type {
            OptionType::Call => {
                -self.r * self.k * self.exp_neg_rt() * self.nd2()
                    + self.q * self.s * self.exp_neg_qt() * self.nd1()
            }
            OptionType::Put => {
                self.r * self.k * self.exp_neg_rt() * self.n_minus_d2()
                    - self.q * self.s * self.exp_neg_qt() * self.n_minus_d1()
            }
        };
        to_valid_or_zero(t1 + t2)
    }

    /// Calculate vega (option type independent)
    pub fn vega(&self) -> f64 {
        let vega = self.s * self.exp_neg_qt() * self.phi_d1() * self.sqrt_t();
        to_valid_or_zero(vega)
    }

    /// Calculate rho
    pub fn rho(&self, option_type: OptionType) -> f64 {
        let rho = match option_type {
            OptionType::Call => self.k * self.t * self.exp_neg_rt() * self.nd2(),
            OptionType::Put => -self.k * self.t * self.exp_neg_rt() * self.n_minus_d2(),
        };
        to_valid_or_zero(rho)
    }

    /// Calculate vanna (option type independent)
    pub fn vanna(&self) -> f64 {
        let vanna = -self.exp_neg_qt() * self.phi_d1() * self.d2() / self.sigma;
        to_valid_or_zero(vanna)
    }

    /// Calculate volga (option type independent)
    pub fn volga(&self) -> f64 {
        let volga =
            self.s * self.exp_neg_qt() * self.phi_d1() * self.sqrt_t() * self.d1() * self.d2()
                / self.sigma;
        to_valid_or_zero(volga)
    }

    /// Calculate charm
    pub fn charm(&self, option_type: OptionType) -> f64 {
        let term1 =
            -self.exp_neg_qt() * self.phi_d1() * (self.r - self.q) / (self.sigma * self.sqrt_t());
        let term2 = self.exp_neg_qt() * self.phi_d1() * self.d2() / (2.0 * self.t);
        let charm = match option_type {
            OptionType::Call => term1 - term2,
            OptionType::Put => term1 + term2,
        };
        to_valid_or_zero(charm)
    }

    /// Calculate speed (option type independent)
    pub fn speed(&self) -> f64 {
        let speed = -self.gamma() * (self.d1() / (self.s * self.sigma_sqrt_t()) + 1.0 / self.s);
        to_valid_or_zero(speed)
    }

    /// Calculate zomma (option type independent)
    pub fn zomma(&self) -> f64 {
        let zomma = self.gamma() * (self.d1() * self.d2() - 1.0) / self.sigma;
        to_valid_or_zero(zomma)
    }
}

// Complete Greeks structure
#[derive(Debug, Clone)]
pub struct Greeks {
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
    pub rho: f64,
    pub vanna: f64,
    pub volga: f64,
    pub charm: f64,
    pub speed: f64,
    pub zomma: f64,
}

impl Default for Greeks {
    fn default() -> Self {
        Self {
            delta: 0.0,
            gamma: 0.0,
            theta: 0.0,
            vega: 0.0,
            rho: 0.0,
            vanna: 0.0,
            volga: 0.0,
            charm: 0.0,
            speed: 0.0,
            zomma: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Price {
    pub price: f64,
    pub intrinsic: f64,
}

#[derive(Debug, Clone)]
pub struct PricingResult {
    pub price: Price,
    pub greeks: Greeks,
}

impl BlackScholesModel {
    /// Calculate all Greeks at once
    pub fn calculate_greeks(&self, option_type: OptionType) -> Greeks {
        Greeks {
            delta: self.delta(option_type),
            gamma: self.gamma(),
            theta: self.theta(option_type),
            vega: self.vega(),
            rho: self.rho(option_type),
            vanna: self.vanna(),
            volga: self.volga(),
            charm: self.charm(option_type),
            speed: self.speed(),
            zomma: self.zomma(),
        }
    }

    /// Calculate price and intrinsic
    pub fn calculate_price(&self, option_type: OptionType) -> Price {
        Price {
            price: self.option_price(option_type),
            intrinsic: self.intrinsic_value(option_type),
        }
    }

    /// Calculate everything at once - most efficient for complete analysis
    pub fn calculate_price_and_greeks(&self, option_type: OptionType) -> PricingResult {
        PricingResult {
            price: self.calculate_price(option_type),
            greeks: self.calculate_greeks(option_type),
        }
    }

    /// Calculate Vanna using finite difference (as in Kotlin version)
    pub fn calculate_vanna_fd(&self, option_type: OptionType) -> f64 {
        const BUMP_SIZE: f64 = 0.01;

        let vol_up = BlackScholesModel::new(
            self.s,
            self.k,
            self.t,
            self.sigma + BUMP_SIZE,
            self.r,
            self.q,
        )
        .expect("Valid parameters");

        let vol_down = BlackScholesModel::new(
            self.s,
            self.k,
            self.t,
            self.sigma - BUMP_SIZE,
            self.r,
            self.q,
        )
        .expect("Valid parameters");

        let delta_up = vol_up.delta(option_type);
        let delta_down = vol_down.delta(option_type);

        (delta_up - delta_down) / (2.0 * BUMP_SIZE)
    }

    /// Calculate Volga using finite difference (as in Kotlin version)
    pub fn calculate_volga_fd(&self, _option_type: OptionType) -> f64 {
        const BUMP_SIZE: f64 = 0.01;

        let vol_up = BlackScholesModel::new(
            self.s,
            self.k,
            self.t,
            self.sigma + BUMP_SIZE,
            self.r,
            self.q,
        )
        .expect("Valid parameters");

        let vol_down = BlackScholesModel::new(
            self.s,
            self.k,
            self.t,
            self.sigma - BUMP_SIZE,
            self.r,
            self.q,
        )
        .expect("Valid parameters");

        let vega_up = vol_up.vega();
        let vega_down = vol_down.vega();

        (vega_up - vega_down) / (2.0 * BUMP_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_pricing() {
        let bs =
            BlackScholesModel::new(100.0, 100.0, 0.25, 0.20, 0.05, 0.0).expect("Valid parameters");

        let call_price = bs.option_price(OptionType::Call);
        let put_price = bs.option_price(OptionType::Put);

        assert!(call_price > 0.0);
        assert!(put_price > 0.0);

        // Put-call parity check: C - P = S*e^(-qT) - K*e^(-rT)
        let lhs = call_price - put_price;
        let rhs = bs.s * bs.exp_neg_qt() - bs.k * bs.exp_neg_rt();
        assert!((lhs - rhs).abs() < 1e-10);
    }

    #[test]
    fn test_greeks_calculation() {
        let bs =
            BlackScholesModel::new(100.0, 100.0, 0.25, 0.20, 0.05, 0.0).expect("Valid parameters");

        let greeks = bs.calculate_greeks(OptionType::Call);

        // Basic sanity checks
        assert!(greeks.delta > 0.0 && greeks.delta < 1.0);
        assert!(greeks.gamma > 0.0);
        assert!(greeks.vega > 0.0);
        assert!(greeks.theta < 0.0); // Calls lose value over time
    }
}
#[test]
fn test_statrs_precision() {
    // Test that statrs provides high precision
    let cdf = norm_cdf(0.0);
    let pdf = norm_pdf(0.0);

    // Check basic values
    assert!((cdf - 0.5).abs() < 1e-14, "CDF(0) should be 0.5");
    assert!(
        (pdf - 0.3989422804014327).abs() < 1e-14,
        "PDF(0) should be 1/√(2π)"
    );
}
