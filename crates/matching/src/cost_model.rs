//! Cost model implementation.
//!
//! Applies deterministic fees or cost models to matched executions.

use domain_types::{Cash, Notional};

/// Calculates deterministic execution costs.
pub trait CostModel {
    /// Calculate total cash required for a gross notional, including costs.
    fn calculate_total_cash(&self, notional: Notional) -> Result<Cash, String>;
}

/// A cost model based on fixed basis points (BPS) of the gross notional.
#[derive(Debug, Clone)]
pub struct FixedBpsCostModel {
    /// Fees in basis points (1 bps = 0.01%)
    pub basis_points: u64,
}

impl FixedBpsCostModel {
    pub fn new(basis_points: u64) -> Self {
        Self { basis_points }
    }
}

impl CostModel for FixedBpsCostModel {
    fn calculate_total_cash(&self, notional: Notional) -> Result<Cash, String> {
        let gross_cash = notional.as_raw();
        // Calculate fee: gross * basis_points / 10000
        let fee = gross_cash
            .checked_mul(self.basis_points)
            .ok_or("Fee calculation overflow")?
            .checked_div(10000)
            .ok_or("Fee division by zero")?; // Actually won't happen because divisor is 10000
            
        let total_cash = gross_cash
            .checked_add(fee)
            .ok_or("Total cash calculation overflow")?;
            
        Ok(Cash::new(total_cash as i128))
    }
}
