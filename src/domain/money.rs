use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Mul};

/// Money stored as integer minor units (cents) to avoid floating-point error.
/// Maps directly to a SQL `BIGINT` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Money = Money(0);

    pub const fn from_cents(cents: i64) -> Self {
        Self(cents)
    }

    pub const fn cents(self) -> i64 {
        self.0
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        Money(self.0 + rhs.0)
    }
}

impl Mul<u32> for Money {
    type Output = Money;
    fn mul(self, qty: u32) -> Money {
        Money(self.0 * i64::from(qty))
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, Add::add)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        write!(f, "{sign}${}.{:02}", abs / 100, abs % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_as_dollars() {
        assert_eq!(Money::from_cents(1234).to_string(), "$12.34");
        assert_eq!(Money::from_cents(5).to_string(), "$0.05");
        assert_eq!(Money::from_cents(-250).to_string(), "-$2.50");
    }

    #[test]
    fn arithmetic() {
        let total: Money = [Money::from_cents(100) * 3, Money::from_cents(50)]
            .into_iter()
            .sum();
        assert_eq!(total, Money::from_cents(350));
    }
}
