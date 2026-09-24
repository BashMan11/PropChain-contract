// Standalone `Treasury` helper (Issue #982), wired into the governance
// contract in #1122. The types are SCALE/StorageLayout compatible so a
// `Treasury` can live directly in the `#[ink(storage)]` struct while still
// being unit-testable as a plain module.

#[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum TreasuryError {
    NotApproved,
    ExceedsSpendLimit,
    InsufficientFunds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Treasury {
    balance: u128,
    spend_limit: u128,
}

impl Treasury {
    pub fn new(balance: u128, spend_limit: u128) -> Self {
        Self {
            balance,
            spend_limit,
        }
    }
    pub fn deposit(&mut self, amount: u128) {
        self.balance = self.balance.saturating_add(amount);
    }
    pub fn balance(&self) -> u128 {
        self.balance
    }
    pub fn spend_limit(&self) -> u128 {
        self.spend_limit
    }
    pub fn set_spend_limit(&mut self, new_limit: u128) {
        self.spend_limit = new_limit;
    }

    /// Non-mutating feasibility check for `release`; lets callers verify the
    /// spend-limit and balance constraints before any state is touched, so a
    /// failed disbursement cannot leave the ledger half-committed.
    pub fn can_release(&self, approved: bool, amount: u128) -> Result<(), TreasuryError> {
        if !approved {
            return Err(TreasuryError::NotApproved);
        }
        if amount > self.spend_limit {
            return Err(TreasuryError::ExceedsSpendLimit);
        }
        if amount > self.balance {
            return Err(TreasuryError::InsufficientFunds);
        }
        Ok(())
    }

    pub fn release(&mut self, approved: bool, amount: u128) -> Result<u128, TreasuryError> {
        self.can_release(approved, amount)?;
        self.balance = self.balance.saturating_sub(amount);
        Ok(amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_within_limit() {
        let mut t = Treasury::new(10_000, 5_000);
        assert_eq!(t.release(true, 3_000).unwrap(), 3_000);
        assert_eq!(t.balance(), 7_000);
    }

    #[test]
    fn exceeds_spend_limit() {
        let mut t = Treasury::new(10_000, 5_000);
        assert_eq!(
            t.release(true, 6_000),
            Err(TreasuryError::ExceedsSpendLimit)
        );
    }

    #[test]
    fn unapproved_proposal_rejected() {
        let mut t = Treasury::new(10_000, 5_000);
        assert_eq!(t.release(false, 100), Err(TreasuryError::NotApproved));
    }

    #[test]
    fn insufficient_funds() {
        let mut t = Treasury::new(100, 5_000);
        assert_eq!(t.release(true, 200), Err(TreasuryError::InsufficientFunds));
    }

    #[test]
    fn deposit_increases_balance() {
        let mut t = Treasury::new(0, 1_000);
        t.deposit(500);
        assert_eq!(t.balance(), 500);
    }
}
