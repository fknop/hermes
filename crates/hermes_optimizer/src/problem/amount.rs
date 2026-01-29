use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Index, IndexMut, Sub, SubAssign},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::utils;

pub trait AmountExpression: Sized {
    fn get(&self, index: usize) -> f64;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0 || self.iter().all(|v| v == 0.0)
    }
    fn iter(&self) -> impl Iterator<Item = f64>
    where
        Self: Sized;
}

type Vector = SmallVec<[f64; 2]>;

// 1. Blanket implementation for References
// This allows &Amount to be used anywhere AmountExpression is required.
impl<T: AmountExpression + Sized> AmountExpression for &T {
    fn get(&self, index: usize) -> f64 {
        (**self).get(index)
    }
    fn len(&self) -> usize {
        (**self).len()
    }
    fn iter(&self) -> impl Iterator<Item = f64>
    where
        Self: Sized,
    {
        (**self).iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Amount(Vector);

impl Amount {
    pub const EMPTY: Amount = Amount(Vector::new_const());

    pub fn empty() -> Self {
        Self::EMPTY
    }

    pub fn with_dimensions(capacity: usize) -> Self {
        let mut vec = SmallVec::with_capacity(capacity);
        vec.resize(capacity, 0.0);
        Amount(vec)
    }

    pub fn from_vec(vec: Vec<f64>) -> Self {
        Amount(SmallVec::from_vec(vec))
    }

    pub fn reset(&mut self) {
        self.0.clear();
    }

    pub fn update(&mut self, other: &Amount) {
        self.0.clone_from(&other.0);
    }

    pub fn update_expr(&mut self, other: impl AmountExpression) {
        self.0.clear();
        self.0.extend(other.iter());
    }

    pub fn update_max(&mut self, other: &Amount) {
        let max_len = self.len().max(other.len());
        self.0.resize(max_len, 0.0);
        for i in 0..max_len {
            self.0[i] = self.get(i).max(other.get(i));
        }
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

impl IndexMut<usize> for Amount {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.0.len() {
            self.0.resize(index + 1, 0.0);
        }

        &mut self.0[index]
    }
}

impl<E: AmountExpression> AddAssign<E> for Amount {
    fn add_assign(&mut self, rhs: E) {
        if self.0.len() < rhs.len() {
            self.0.resize(rhs.len(), 0.0);
        }

        for (a, b) in self.0.iter_mut().zip(rhs.iter()) {
            *a += b;
        }
    }
}

impl<E: AmountExpression> SubAssign<E> for Amount {
    fn sub_assign(&mut self, rhs: E) {
        if self.0.len() < rhs.len() {
            self.0.resize(rhs.len(), 0.0);
        }
        for (a, b) in self.0.iter_mut().zip(rhs.iter()) {
            *a -= b;
        }
    }
}

impl<'a, 'b> Add<&'b Amount> for &'a Amount {
    type Output = AmountSum<&'a Amount, &'b Amount>;
    fn add(self, rhs: &'b Amount) -> Self::Output {
        AmountSum { lhs: self, rhs }
    }
}
impl<'a, 'b> Sub<&'b Amount> for &'a Amount {
    type Output = AmountSub<&'a Amount, &'b Amount>;
    fn sub(self, rhs: &'b Amount) -> Self::Output {
        AmountSub { lhs: self, rhs }
    }
}

impl<A> PartialEq<A> for Amount
where
    A: AmountExpression,
{
    fn eq(&self, other: &A) -> bool {
        if self.len() != other.len() {
            // Edge case where it can be 0.0 or empty vec
            if self.is_empty() && other.is_empty() {
                return true;
            }

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

impl Eq for Amount {}

#[derive(Debug, Clone)]
pub struct AmountSum<LHS, RHS> {
    pub lhs: LHS,
    pub rhs: RHS,
}

impl<LHS, RHS> AmountExpression for AmountSum<LHS, RHS>
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
        utils::zip_longest::zip_longest(self.lhs.iter(), self.rhs.iter())
            .map(|(a, b)| a.unwrap_or(0.0) + b.unwrap_or(0.0))
    }
}

impl<LHS, RHS> PartialEq<Amount> for AmountSum<LHS, RHS>
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    fn eq(&self, other: &Amount) -> bool {
        other.eq(self)
    }
}

impl<LHS, RHS> PartialOrd<Amount> for AmountSum<LHS, RHS>
where
    LHS: AmountExpression,
    RHS: AmountExpression,
{
    fn partial_cmp(&self, other: &Amount) -> Option<std::cmp::Ordering> {
        other.partial_cmp(self).map(|o| o.reverse())
    }
}

#[derive(Debug, Clone)]
pub struct AmountSub<L, R> {
    lhs: L,
    rhs: R,
}

impl<L, R> AmountExpression for AmountSub<L, R>
where
    L: AmountExpression,
    R: AmountExpression,
{
    fn len(&self) -> usize {
        self.lhs.len().max(self.rhs.len())
    }

    fn get(&self, index: usize) -> f64 {
        self.lhs.get(index) - self.rhs.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = f64> {
        utils::zip_longest::zip_longest(self.lhs.iter(), self.rhs.iter())
            .map(|(a, b)| a.unwrap_or(0.0) - b.unwrap_or(0.0))
    }
}

// Allow AmountSub == Amount
impl<L, R> PartialEq<Amount> for AmountSub<L, R>
where
    L: AmountExpression,
    R: AmountExpression,
{
    fn eq(&self, other: &Amount) -> bool {
        other.eq(self)
    }
}

impl<A> PartialOrd<A> for Amount
where
    A: AmountExpression,
{
    fn partial_cmp(&self, other: &A) -> Option<Ordering> {
        // Lexicographical-style comparison over the values?
        // Your original logic compared index by index.
        let common_len = self.len().max(other.len());
        for i in 0..common_len {
            let s = self.get(i);
            let o = other.get(i);
            if s < o {
                return Some(Ordering::Less);
            }
            if s > o {
                return Some(Ordering::Greater);
            }
        }
        Some(Ordering::Equal)
    }
}

impl<L, R> From<AmountSum<L, R>> for Amount
where
    L: AmountExpression,
    R: AmountExpression,
{
    fn from(val: AmountSum<L, R>) -> Self {
        // We know the exact length, so we can pre-allocate
        let mut vec = SmallVec::with_capacity(val.len());
        vec.extend(val.iter());
        Amount(vec)
    }
}

// 3. Implement for AmountSub
impl<L, R> From<AmountSub<L, R>> for Amount
where
    L: AmountExpression,
    R: AmountExpression,
{
    fn from(val: AmountSub<L, R>) -> Self {
        let mut vec = SmallVec::with_capacity(val.len());
        vec.extend(val.iter());
        Amount(vec)
    }
}

// Macro to implement Add<RHS> for LHS where output is AmountSum<LHS, RHS>
macro_rules! impl_add_mix {
    ($lhs:ty, $rhs:ty) => {
        impl<'a, L, R> Add<$rhs> for $lhs
        where
            L: AmountExpression,
            R: AmountExpression,
        {
            type Output = AmountSum<$lhs, $rhs>;
            fn add(self, rhs: $rhs) -> Self::Output {
                AmountSum { lhs: self, rhs }
            }
        }
    };
}

// Macro to implement Sub<RHS> for LHS where output is AmountSub<LHS, RHS>
macro_rules! impl_sub_mix {
    ($lhs:ty, $rhs:ty) => {
        impl<'a, L, R> Sub<$rhs> for $lhs
        where
            L: AmountExpression,
            R: AmountExpression,
        {
            type Output = AmountSub<$lhs, $rhs>;
            fn sub(self, rhs: $rhs) -> Self::Output {
                AmountSub { lhs: self, rhs }
            }
        }
    };
}

// Register combinations
impl_add_mix!(AmountSum<L, R>, &'a Amount);
impl_add_mix!(AmountSub<L, R>, &'a Amount);
impl_add_mix!(AmountSum<L, R>, AmountSum<L, R>); // Self mix
impl_add_mix!(AmountSum<L, R>, AmountSub<L, R>); // Cross mix
impl_add_mix!(AmountSub<L, R>, AmountSum<L, R>); // Cross mix
impl_add_mix!(AmountSub<L, R>, AmountSub<L, R>); // Self mix
impl_add_mix!(&'a Amount, AmountSum<L, R>);
impl_add_mix!(&'a Amount, AmountSub<L, R>);

impl_sub_mix!(AmountSum<L, R>, &'a Amount);
impl_sub_mix!(AmountSub<L, R>, &'a Amount);
impl_sub_mix!(AmountSum<L, R>, AmountSum<L, R>);
impl_sub_mix!(AmountSum<L, R>, AmountSub<L, R>);
impl_sub_mix!(AmountSub<L, R>, AmountSum<L, R>);
impl_sub_mix!(AmountSub<L, R>, AmountSub<L, R>);
impl_sub_mix!(&'a Amount, AmountSum<L, R>);
impl_sub_mix!(&'a Amount, AmountSub<L, R>);

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

    #[test]
    fn test_nesting_mix() {
        let a = Amount::from_vec(vec![100.0]);
        let b = Amount::from_vec(vec![50.0]);
        let c = Amount::from_vec(vec![25.0]);

        // ((a - b) + c)
        let calc = (&a - &b) + &c;
        assert_eq!(calc.get(0), 75.0);

        // ((a + b) - c)
        let calc2 = (&a + &b) - &c;
        assert_eq!(calc2.get(0), 125.0);
    }

    #[test]
    fn test_nesting_add() {
        let a = Amount::from_vec(vec![10.0]);
        let b = Amount::from_vec(vec![10.0]);
        let c = Amount::from_vec(vec![10.0]);

        // (a + b) is AmountSum<&Amount, &Amount>
        // (a + b) + c is AmountSum<AmountSum<&Amount, &Amount>, &Amount>
        let sum = &a + &b + &c;

        assert_eq!(sum.get(0), 30.0);

        let result: Amount = Amount::from(sum);
        assert_eq!(result.get(0), 30.0);
    }

    #[test]
    fn test_amount_update() {
        let mut a = Amount::from_vec(vec![1.0, 2.0, 3.0]);
        let b = Amount::from_vec(vec![4.0, 5.0]);

        a.update(&b);

        assert_eq!(a.len(), 2);
        assert_eq!(a.get(0), 4.0);
        assert_eq!(a.get(1), 5.0);
    }

    #[test]
    fn test_amount_update_expr() {
        let mut a = Amount::empty();
        let b = Amount::from_vec(vec![1.0, 2.0, 3.0]);
        let c = Amount::from_vec(vec![4.0, 5.0, 4.0]);
        let d = Amount::from_vec(vec![4.0, 5.0, 4.0]);

        a.update_expr(&b + &c + &d);

        assert_eq!(a.len(), 3);
        assert_eq!(a, Amount::from_vec(vec![9.0, 12.0, 11.0]));
    }

    #[test]
    fn test_amount_update_expr_with_empty() {
        let mut a = Amount::empty();
        let empty = Amount::empty();
        let b = Amount::from_vec(vec![1.0, 2.0, 3.0]);
        let c = Amount::from_vec(vec![4.0, 5.0, 4.0]);
        let d = Amount::from_vec(vec![4.0, 5.0, 4.0]);

        a.update_expr(&empty + &b + &c + &d);

        assert_eq!(a.len(), 3);
        assert_eq!(a, Amount::from_vec(vec![9.0, 12.0, 11.0]));
    }

    #[test]
    fn test_with_dimensions() {
        let demand = Amount::with_dimensions(1);
        assert_eq!(demand.len(), 1);
    }

    #[test]
    fn test_eq() {
        let a = Amount::from_vec(vec![1.0, 2.0, 3.0]);
        let b = Amount::from_vec(vec![1.0, 2.0, 3.0]);
        let c = Amount::from_vec(vec![1.0, 2.0, 4.0]);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
