use std::ops::{Add, AddAssign, Index, Sub, SubAssign};

use smallvec::SmallVec;

trait GetAmount {
    fn get(&self, index: usize) -> f64;
    fn len(&self) -> usize;
    fn empty(&self) -> bool {
        self.len() == 0
    }
}

type Vector = SmallVec<[f64; 2]>;

#[derive(Debug, Clone)]
struct Amount(Vector);

impl Amount {
    fn from_vec(vec: Vec<f64>) -> Self {
        Amount(SmallVec::from_vec(vec))
    }
}

impl GetAmount for Amount {
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn get(&self, index: usize) -> f64 {
        self.0.get(index).cloned().unwrap_or(0.0)
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
    type Output = AmountSum<'a>;

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
    A: GetAmount,
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
    A: GetAmount,
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
struct AmountSum<'a> {
    lhs: &'a Amount,
    rhs: &'a Amount,
}

impl GetAmount for AmountSum<'_> {
    fn len(&self) -> usize {
        self.lhs.len().max(self.rhs.len())
    }

    fn get(&self, index: usize) -> f64 {
        self.lhs.get(index) + self.rhs.get(index)
    }
}

impl PartialEq<Amount> for AmountSum<'_> {
    fn eq(&self, other: &Amount) -> bool {
        other.eq(self)
    }
}

impl PartialOrd<Amount> for AmountSum<'_> {
    fn partial_cmp(&self, other: &Amount) -> Option<std::cmp::Ordering> {
        other.partial_cmp(self).map(|o| o.reverse())
    }
}

#[derive(Debug, Clone)]
struct AmountSub<'a> {
    lhs: &'a Amount,
    rhs: &'a Amount,
}

impl GetAmount for AmountSub<'_> {
    fn len(&self) -> usize {
        self.lhs.len().max(self.rhs.len())
    }

    fn get(&self, index: usize) -> f64 {
        self.lhs.get(index) - self.rhs.get(index)
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
