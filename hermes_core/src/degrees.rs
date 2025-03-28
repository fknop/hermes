// This below comes from osmio

/// How many nanodegrees are represented by each unit in [`Lat::inner()`].
/// We use the same internal precision as OpenStreetMap.org, 100 nanodegrees.
pub const COORD_PRECISION_NANOS: i32 = 100;

/// Number of internal units (as returned from [`Lat::inner`]) in one degree.
///
/// See [`COORD_PRECISION_NANOS`].
pub const COORD_SCALE_FACTOR: f64 = (1_000_000_000 / COORD_PRECISION_NANOS) as f64;

#[derive(PartialEq, Copy, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Degrees(i32);

impl Degrees {
    pub fn degrees(&self) -> f64 {
        self.0 as f64 / COORD_SCALE_FACTOR
    }

    pub fn nanos(&self) -> i32 {
        self.0
    }
}

impl From<Degrees> for f64 {
    fn from(value: Degrees) -> Self {
        value.degrees()
    }
}

impl From<i32> for Degrees {
    fn from(value: i32) -> Self {
        Degrees(value)
    }
}

impl TryFrom<f64> for Degrees {
    type Error = ParseDegreesError;
    fn try_from(val: f64) -> Result<Degrees, Self::Error> {
        match (val * COORD_SCALE_FACTOR).round() {
            x if x > (i32::MAX as f64) => Err(ParseDegreesError::TooLarge(x)),
            x if x < (i32::MIN as f64) => Err(ParseDegreesError::TooSmall(x)),
            x => Ok(Self(x as i32)),
        }
    }
}

impl std::fmt::Debug for Degrees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}°", self.degrees())
    }
}

impl std::fmt::Display for Degrees {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}°", self.degrees())
    }
}

#[derive(Debug)]
pub enum ParseDegreesError {
    ParseFloatError(std::num::ParseFloatError),
    TooLarge(f64),
    TooSmall(f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_degrees_conversion() {
        let d = Degrees(1_000_000);
        assert_eq!(d.degrees(), 0.1);

        let d = Degrees(10_000_000);
        assert_eq!(d.degrees(), 1.0);

        let d = Degrees(-1_000_000);
        assert_eq!(d.degrees(), -0.1);
    }

    #[test]
    fn test_f64_conversion() {
        let d: f64 = Degrees(1_000_000).into();
        assert_eq!(d, 0.1);

        let d = Degrees::try_from(0.1f64).unwrap();
        assert_eq!(d.0, 1_000_000);

        let degrees = Degrees::try_from(4.5872838).unwrap();
        assert_eq!(degrees.0, 45_872_838);

        let degrees = Degrees::try_from(4.58728393).unwrap();
        // Should cut the precision
        assert_eq!(degrees.0, 45_872_839);
    }

    #[test]
    fn test_i32_conversion() {
        let d = Degrees::from(1_000_000);
        assert_eq!(d.0, 1_000_000);
    }

    #[test]
    fn test_out_of_bounds() {
        let too_large = f64::MAX;
        assert!(matches!(
            Degrees::try_from(too_large),
            Err(ParseDegreesError::TooLarge(_))
        ));

        let too_small = f64::MIN;
        assert!(matches!(
            Degrees::try_from(too_small),
            Err(ParseDegreesError::TooSmall(_))
        ));
    }

    #[test]
    fn test_display() {
        let d = Degrees(1_000_000);
        assert_eq!(format!("{}", d), "0.1°");
        assert_eq!(format!("{:?}", d), "0.1°");
    }
}
