use std::{
    cmp::Ordering,
    fmt,
    marker::PhantomData,
    ops::{Add, Div, Sub},
};

pub trait DistanceUnit: Copy + Eq {
    const NAME: &'static str;
    const NANOMETERS_IN_UNIT: i64;
}

#[derive(Debug, Clone, Copy, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Distance<T: DistanceUnit> {
    nm: i64,
    unit: PhantomData<T>,
}

macro_rules! create_distance_unit {
    ($struct_name:ident, $string_name:expr , $nm_conv:expr) => {
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub struct $struct_name; // unit-like struct

        impl DistanceUnit for $struct_name {
            const NAME: &'static str = $string_name;
            const NANOMETERS_IN_UNIT: i64 = $nm_conv;
        }

        impl Distance<$struct_name> {
            pub fn new(value: i64) -> Distance<$struct_name> {
                Distance {
                    nm: value * $struct_name::NANOMETERS_IN_UNIT,
                    unit: PhantomData,
                }
            }

            #[inline(always)]
            pub fn value(&self) -> f64 {
                (self.nm as f64) / ($struct_name::NANOMETERS_IN_UNIT as f64)
            }
        }
    };
}

create_distance_unit!(Meters, "meter", 1_000_000_000);
create_distance_unit!(Kilometers, "kilometer", 1_000_000_000_000);

impl<T> From<Distance<T>> for f64
where
    T: DistanceUnit,
{
    fn from(value: Distance<T>) -> Self {
        (value.nm as f64) / T::NANOMETERS_IN_UNIT as f64
    }
}

impl<T> fmt::Display for Distance<T>
where
    T: DistanceUnit,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value: f64 = (self.nm as f64) / (T::NANOMETERS_IN_UNIT as f64);

        write!(
            f,
            "{} {}{}",
            value,
            T::NAME,
            match value {
                1_f64 => "",
                _ => "s",
            }
        )
    }
}

impl<T> Ord for Distance<T>
where
    T: DistanceUnit,
{
    fn cmp(&self, other: &Distance<T>) -> Ordering {
        self.nm.cmp(&other.nm)
    }
}

impl<T1, T2> PartialEq<Distance<T2>> for Distance<T1>
where
    T1: DistanceUnit,
    T2: DistanceUnit,
{
    fn eq(&self, other: &Distance<T2>) -> bool {
        self.nm == other.nm
    }
}

// implement PartialORd for ordering Lengths with different units
impl<T1, T2> PartialOrd<Distance<T2>> for Distance<T1>
where
    T1: DistanceUnit,
    T2: DistanceUnit,
{
    fn partial_cmp(&self, other: &Distance<T2>) -> Option<Ordering> {
        Some(self.nm.cmp(&other.nm))
    }
}

impl<T> From<f64> for Distance<T>
where
    T: DistanceUnit,
{
    fn from(value: f64) -> Self {
        Distance {
            nm: (value * (T::NANOMETERS_IN_UNIT as f64)).round() as i64,
            unit: PhantomData,
        }
    }
}

impl<T> From<i64> for Distance<T>
where
    T: DistanceUnit,
{
    fn from(value: i64) -> Self {
        Distance {
            nm: value * T::NANOMETERS_IN_UNIT,
            unit: PhantomData,
        }
    }
}

impl<T1, T2> Add<Distance<T2>> for Distance<T1>
where
    T1: DistanceUnit,
    T2: DistanceUnit,
{
    type Output = Distance<T1>;

    fn add(self, other: Distance<T2>) -> Distance<T1> {
        Distance {
            nm: self.nm + other.nm,
            unit: PhantomData,
        }
    }
}

impl<T1, T2> Sub<Distance<T2>> for Distance<T1>
where
    T1: DistanceUnit,
    T2: DistanceUnit,
{
    type Output = Distance<T1>;

    fn sub(self, other: Distance<T2>) -> Distance<T1> {
        Distance {
            nm: self.nm - other.nm,
            unit: PhantomData,
        }
    }
}

impl<T1, T2> Div<Distance<T2>> for Distance<T1>
where
    T1: DistanceUnit,
    T2: DistanceUnit,
{
    type Output = f64;

    fn div(self, other: Distance<T2>) -> f64 {
        (self.nm as f64) / (other.nm as f64)
    }
}

macro_rules! meters {
    ($num:expr) => {
        crate::distance::Distance::<crate::distance::Meters>::from($num)
    };
}

macro_rules! kilometers {
    ($num:expr) => {
        crate::distance::Distance::<crate::distance::Kilometers>::from($num)
    };
}

pub(crate) use meters;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_distances_together() {
        let result = meters!(10) + kilometers!(1);
        assert_eq!(result, meters!(1010));
    }

    #[test]
    fn should_divide_distance() {
        assert_eq!(meters!(100) / meters!(10), 10.0);
    }

    #[test]
    fn value() {
        assert_eq!(meters!(100).value(), 100.0);
    }
}
