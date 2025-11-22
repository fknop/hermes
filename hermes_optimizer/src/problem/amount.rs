use std::ops::{Add, AddAssign, Index, Sub, SubAssign};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub trait AmountExpression {
    fn get(&self, index: usize) -> f64;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter(&self) -> impl Iterator<Item = f64>;
}

type Vector = SmallVec<[f64; 2]>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount(Vector);

impl Amount {
    pub const EMPTY: Amount = Amount(Vector::new_const());

    pub fn empty() -> Self {
        Self::EMPTY
    }

    pub fn from_vec(vec: Vec<f64>) -> Self {
        Amount(SmallVec::from_vec(vec))
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl AmountExpression for Amount {
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn get(&self, index: usize) -> f64 {
        self.0.get(index).cloned().unwrap_or(0.0)
    }

    fn iter(&self) -> impl Iterator<Item = f64> {
        self.0.iter().cloned()
    }
}

impl Index<usize> for Amount {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl AddAssign<&Amount> for Amount {
    fn add_assign(&mut self, rhs: &Amount) {
        if self.0.len() < rhs.0.len() {
            self.0.resize(rhs.0.len(), 0.0);
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a += *b;
        }
    }
}

impl SubAssign<&Amount> for Amount {
    fn sub_assign(&mut self, rhs: &Amount) {
        if self.0.len() < rhs.0.len() {
            self.0.resize(rhs.0.len(), 0.0);
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a -= *b;
        }
    }
}

impl<'a> Add<&'a Amount> for &'a Amount {
    type Output = AmountSum<'a, Amount, Amount>;

    fn add(self, rhs: &'a Amount) -> Self::Output {
        AmountSum { lhs: self, rhs }
    }
}

impl<'a> Sub<&'a Amount> for &'a Amount {
    type Output = AmountSub<'a>;

    fn sub(self, rhs: &'a Amount) -> Self::Output {
        AmountSub { lhs: self, rhs }
    }
}

impl<A> PartialEq<A> for Amount
where
    A: AmountExpression,
{
    fn eq(&self, other: &A) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for i in 0..self.len().max(other.len()) {
            let self_value = self.get(i);
            let other_value = other.get(i);
            if self_value != other_value {
                return false;
            }
        }
        true
    }
}

impl<A> PartialOrd<A> for Amount
where
    A: AmountExpression,
{
    fn partial_cmp(&self, other: &A) -> Option<std::cmp::Ordering> {
        for i in 0..self.len().max(other.len()) {
            let self_value = self.get(i);
            let other_value = other.get(i);
            if self_value < other_value {
                return Some(std::cmp::Ordering::Less);
            } else if self_value > other_value {
                return Some(std::cmp::Ordering::Greater);
            }
        }
        Some(std::cmp::Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
pub struct AmountSum<'a, LHS, RHS>
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    lhs: &'a LHS,
    rhs: &'a RHS,
}

impl<'a, LHS, RHS> AmountExpression for AmountSum<'a, LHS, RHS>
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    fn len(&self) -> usize {
        self.lhs.len().max(self.rhs.len())
    }

    fn get(&self, index: usize) -> f64 {
        self.lhs.get(index) + self.rhs.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = f64> {
        self.lhs.iter().zip(self.rhs.iter()).map(|(a, b)| a + b)
    }
}

impl<'a, LHS, RHS> PartialEq<Amount> for AmountSum<'a, LHS, RHS>
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    fn eq(&self, other: &Amount) -> bool {
        other.eq(self)
    }
}

impl<'a, LHS, RHS> PartialOrd<Amount> for AmountSum<'a, LHS, RHS>
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    fn partial_cmp(&self, other: &Amount) -> Option<std::cmp::Ordering> {
        other.partial_cmp(self).map(|o| o.reverse())
    }
}

impl<'a, LHS, RHS> From<AmountSum<'a, LHS, RHS>> for Amount
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    fn from(val: AmountSum<'a, LHS, RHS>) -> Self {
        let mut vec = SmallVec::with_capacity(val.len());
        for i in 0..val.len() {
            vec.push(val.get(i));
        }
        Amount(vec)
    }
}

#[derive(Debug, Clone)]
pub struct AmountSub<'a> {
    lhs: &'a Amount,
    rhs: &'a Amount,
}

impl AmountExpression for AmountSub<'_> {
    fn len(&self) -> usize {
        self.lhs.len().max(self.rhs.len())
    }

    fn get(&self, index: usize) -> f64 {
        self.lhs.get(index) - self.rhs.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = f64> {
        self.lhs.iter().zip(self.rhs.iter()).map(|(a, b)| a - b)
    }
}

impl PartialEq<Amount> for AmountSub<'_> {
    fn eq(&self, other: &Amount) -> bool {
        other.eq(self)
    }
}

impl PartialOrd<Amount> for AmountSub<'_> {
    fn partial_cmp(&self, other: &Amount) -> Option<std::cmp::Ordering> {
        other.partial_cmp(self).map(|o| o.reverse())
    }
}

impl From<AmountSub<'_>> for Amount {
    fn from(val: AmountSub<'_>) -> Self {
        let mut vec = SmallVec::with_capacity(val.len());
        for i in 0..val.len() {
            vec.push(val.get(i));
        }
        Amount(vec)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_amount_add_assign() {
        let mut a = Amount::from_vec(vec![10.0, 20.0]);
        let b = Amount::from_vec(vec![5.0, 15.0, 25.0]);

        a += &b;

        assert_eq!(a.get(0), 15.0);
        assert_eq!(a.get(1), 35.0);
        assert_eq!(a.get(2), 25.0);
    }

    #[test]
    fn test_amount_sub_assign() {
        let mut a = Amount::from_vec(vec![10.0, 20.0, 30.0]);
        let b = Amount::from_vec(vec![5.0, 15.0]);

        a -= &b;

        assert_eq!(a.get(0), 5.0);
        assert_eq!(a.get(1), 5.0);
        assert_eq!(a.get(2), 30.0);
    }

    #[test]
    fn test_amount_sum_equality() {
        let a = Amount::from_vec(vec![10.0, 20.0]);
        let b = Amount::from_vec(vec![5.0, 15.0, 25.0]);

        let sum = &a + &b;

        let expected = Amount::from_vec(vec![15.0, 35.0, 25.0]);

        assert_eq!(sum, expected);
    }

    #[test]
    fn test_amount_sum_partial_ord_eq() {
        let a = Amount::from_vec(vec![10.0, 20.0]);
        let b = Amount::from_vec(vec![5.0, 15.0, 25.0]);
        let sum = &a + &b;
        let expected = Amount::from_vec(vec![15.0, 35.0, 25.0]);
        assert!(sum == expected);
    }

    #[test]
    fn test_amount_sum_partial_ord_less() {
        let a = Amount::from_vec(vec![10.0, 20.0]);
        let b = Amount::from_vec(vec![5.0, 15.0]);
        let sum = &a + &b;
        let expected = Amount::from_vec(vec![25.0, 35.0]);
        assert!(sum < expected);
    }

    #[test]
    fn test_amount_sum_partial_ord_greater() {
        let a = Amount::from_vec(vec![10.0, 20.0]);
        let b = Amount::from_vec(vec![5.0, 15.0, 25.0]);
        let sum = &a + &b;
        let expected = Amount::from_vec(vec![25.0, 35.0, 25.0]);
        assert!(expected > sum);
    }
}
