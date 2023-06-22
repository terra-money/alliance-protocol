use cosmwasm_std::{Decimal, DecimalRangeExceeded, StdError, Uint128};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::fmt::Write;
use std::ops::{Add, AddAssign, Div, Mul, Sub};
use std::str::FromStr;

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub enum Sign {
    #[default]
    Positive,
    Negative,
}

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct SignedDecimal {
    value: Decimal,
    sign: Sign,
}

impl SignedDecimal {
    pub fn from_atomics(
        atomics: impl Into<Uint128>,
        decimal_places: u32,
        sign: Sign,
    ) -> Result<Self, DecimalRangeExceeded> {
        let value = Decimal::from_atomics(atomics.into(), decimal_places)?;
        Ok(Self { value, sign })
    }

    pub fn from_decimal(decimal: Decimal, sign: Sign) -> Self {
        if decimal.is_zero() {
            return Self {
                value: Decimal::zero(),
                sign: Sign::Positive,
            };
        }
        Self {
            value: decimal,
            sign,
        }
    }

    pub fn is_positive(&self) -> bool {
        match self.sign {
            Sign::Positive => true,
            Sign::Negative => false,
        }
    }

    pub fn is_negative(&self) -> bool {
        match self.sign {
            Sign::Positive => false,
            Sign::Negative => true,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }

    pub fn zero() -> Self {
        Self {
            value: Decimal::zero(),
            sign: Sign::Positive,
        }
    }
}

impl Add for SignedDecimal {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_negative() && rhs.is_negative() {
            return Self::from_decimal(self.value + rhs.value, Sign::Negative);
        }
        if self.is_negative() && rhs.is_positive() {
            if self.value > rhs.value {
                return Self::from_decimal(self.value - rhs.value, Sign::Negative);
            } else {
                return Self::from_decimal(rhs.value - self.value, Sign::Positive);
            }
        }
        if self.is_positive() && rhs.is_negative() {
            if self.value > rhs.value {
                return Self::from_decimal(self.value - rhs.value, Sign::Positive);
            } else {
                return Self::from_decimal(rhs.value - self.value, Sign::Negative);
            }
        }
        Self::from_decimal(self.value + rhs.value, Sign::Positive)
    }
}

impl Add<Decimal> for SignedDecimal {
    type Output = Self;
    fn add(self, rhs: Decimal) -> Self::Output {
        self + Self::from_decimal(rhs, Sign::Positive)
    }
}

impl Sub<Decimal> for SignedDecimal {
    type Output = Self;
    fn sub(self, rhs: Decimal) -> Self::Output {
        self - Self::from_decimal(rhs, Sign::Positive)
    }
}

impl AddAssign for SignedDecimal {
    fn add_assign(&mut self, rhs: SignedDecimal) {
        *self = *self + rhs;
    }
}

impl AddAssign<Decimal> for SignedDecimal {
    fn add_assign(&mut self, rhs: Decimal) {
        *self = *self + rhs;
    }
}

impl Sub for SignedDecimal {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_negative() && rhs.is_negative() {
            if self.value > rhs.value {
                return Self::from_decimal(self.value - rhs.value, Sign::Negative);
            } else {
                return Self::from_decimal(rhs.value - self.value, Sign::Positive);
            }
        }
        if self.is_negative() && rhs.is_positive() {
            return Self::from_decimal(self.value + rhs.value, Sign::Negative);
        }
        if self.is_positive() && rhs.is_negative() {
            return Self::from_decimal(self.value + rhs.value, Sign::Positive);
        }
        if self.value > rhs.value {
            Self::from_decimal(self.value - rhs.value, Sign::Positive)
        } else {
            Self::from_decimal(rhs.value - self.value, Sign::Negative)
        }
    }
}

impl Mul for SignedDecimal {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.is_negative() && rhs.is_negative() {
            return Self::from_decimal(self.value * rhs.value, Sign::Positive);
        }
        if self.is_negative() && rhs.is_positive() {
            return Self::from_decimal(self.value * rhs.value, Sign::Negative);
        }
        if self.is_positive() && rhs.is_negative() {
            return Self::from_decimal(self.value * rhs.value, Sign::Negative);
        }
        Self::from_decimal(self.value * rhs.value, Sign::Positive)
    }
}

impl Mul<Decimal> for SignedDecimal {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self::Output {
        self * Self::from_decimal(rhs, Sign::Positive)
    }
}

impl Div for SignedDecimal {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if self.is_negative() && rhs.is_negative() {
            return Self::from_decimal(self.value / rhs.value, Sign::Positive);
        }
        if self.is_negative() && rhs.is_positive() {
            return Self::from_decimal(self.value / rhs.value, Sign::Negative);
        }
        if self.is_positive() && rhs.is_negative() {
            return Self::from_decimal(self.value / rhs.value, Sign::Negative);
        }
        Self::from_decimal(self.value / rhs.value, Sign::Positive)
    }
}

impl Div<Decimal> for SignedDecimal {
    type Output = Self;
    fn div(self, rhs: Decimal) -> Self::Output {
        self / Self::from_decimal(rhs, Sign::Positive)
    }
}

impl fmt::Debug for SignedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SignedDecimal({})", self)
    }
}

impl fmt::Display for SignedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_negative() && self.value != Decimal::zero() {
            f.write_char('-')?;
        }
        let _ = f.write_str(&self.value.to_string());
        Ok(())
    }
}

impl FromStr for SignedDecimal {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sign = if s.starts_with('-') {
            Sign::Negative
        } else {
            Sign::Positive
        };
        let value = Decimal::from_str(s.trim_start_matches('-'))?;
        Ok(Self { value, sign })
    }
}

impl Serialize for SignedDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SignedDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl<'de> de::Visitor<'de> for DecimalVisitor {
    type Value = SignedDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match SignedDecimal::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::signed_decimal::{Sign, SignedDecimal};
    use cosmwasm_std::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_from_to_str() {
        let test_cases = [
            (
                SignedDecimal::from_decimal(Decimal::zero(), Sign::Positive),
                "0",
            ),
            (
                SignedDecimal::from_decimal(Decimal::from_str("1.1").unwrap(), Sign::Negative),
                "-1.1",
            ),
            (
                SignedDecimal::from_decimal(Decimal::from_str("1.1").unwrap(), Sign::Positive),
                "1.1",
            ),
        ];
        for (input, expected) in test_cases.iter() {
            assert_eq!(input.to_string(), *expected);
            assert_eq!(SignedDecimal::from_str(expected).unwrap(), *input);
        }
    }

    #[test]
    fn test_add_operations() {
        let test_cases = vec![
            ("1.0", "1.0", "2.0"),
            ("1.0", "-1.0", "0.0"),
            ("-1.0", "1.0", "0.0"),
            ("-1.0", "-1.0", "-2.0"),
            ("1.1", "1.1", "2.2"),
        ];
        for test_case in test_cases {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    + SignedDecimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }

        let test_cases_for_decimal = vec![
            ("1.0", "1.0", "2.0"),
            ("-1.0", "1.0", "0.0"),
            ("1.1", "1.1", "2.2"),
        ];
        for test_case in test_cases_for_decimal {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    + Decimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }
    }

    #[test]
    fn test_sub_operations() {
        let test_cases = vec![
            ("1.0", "1.0", "0.0"),
            ("1.0", "-1.0", "2.0"),
            ("-1.0", "1.0", "-2.0"),
            ("-1.0", "-1.0", "0.0"),
            ("1.1", "1.1", "0.0"),
        ];
        for test_case in test_cases {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    - SignedDecimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }

        let test_cases_for_decimal = vec![
            ("1.0", "1.0", "0.0"),
            ("-1.0", "1.0", "-2.0"),
            ("1.1", "1.1", "0.0"),
        ];
        for test_case in test_cases_for_decimal {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    - Decimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }
    }

    #[test]
    fn test_mul_operations() {
        let test_cases = vec![
            ("1.0", "1.0", "1.0"),
            ("1.0", "-1.0", "-1.0"),
            ("-1.0", "1.0", "-1.0"),
            ("-1.0", "-1.0", "1.0"),
            ("1.1", "1.1", "1.21"),
        ];
        for test_case in test_cases {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    * SignedDecimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }

        let test_cases_for_decimal = vec![
            ("1.0", "1.0", "1.0"),
            ("-1.0", "1.0", "-1.0"),
            ("1.1", "1.1", "1.21"),
        ];
        for test_case in test_cases_for_decimal {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    * Decimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }
    }

    #[test]
    fn test_div_operations() {
        let test_cases = vec![
            ("1.0", "1.0", "1.0"),
            ("1.0", "-1.0", "-1.0"),
            ("-1.0", "1.0", "-1.0"),
            ("-1.0", "-1.0", "1.0"),
            ("1.1", "1.1", "1.0"),
        ];
        for test_case in test_cases {
            assert_eq!(
                SignedDecimal::from_str(test_case.0).unwrap()
                    / SignedDecimal::from_str(test_case.1).unwrap(),
                SignedDecimal::from_str(test_case.2).unwrap()
            );
        }
    }
}
