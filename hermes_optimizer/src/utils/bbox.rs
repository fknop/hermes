use std::f64;

pub struct BBox {
    min: geo::Coord<f64>,
    max: geo::Coord<f64>,
}

impl BBox {
    pub fn extend<C>(&mut self, coord: C)
    where
        C: Into<geo::Coord<f64>>,
    {
        let coord = coord.into();
        self.min.x = self.min.x.min(coord.x);
        self.min.y = self.min.y.min(coord.y);
        self.max.x = self.max.x.max(coord.x);
        self.max.y = self.max.y.max(coord.y);
    }

    pub fn intersects(&self, other: &BBox) -> bool {
        other.min.x <= self.max.x
            && other.min.y <= self.max.y
            && self.min.x <= other.max.x
            && self.min.y <= other.max.y
    }
}

impl Default for BBox {
    fn default() -> Self {
        BBox {
            min: geo::Coord {
                x: f64::MAX,
                y: f64::MAX,
            },
            max: geo::Coord {
                x: f64::MIN,
                y: f64::MIN,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BBox;
    use geo::Coord;

    #[test]
    fn test_bbox_extend() {
        let mut bbox = BBox::default();

        bbox.extend(Coord { x: 0.0, y: 0.0 });

        assert_eq!(bbox.min, Coord { x: 0.0, y: 0.0 });
        assert_eq!(bbox.max, Coord { x: 0.0, y: 0.0 });

        bbox.extend(Coord { x: 1.0, y: 1.0 });
        assert_eq!(bbox.min, Coord { x: 0.0, y: 0.0 });
        assert_eq!(bbox.max, Coord { x: 1.0, y: 1.0 });

        bbox.extend(Coord { x: -4.0, y: 5.0 });
        assert_eq!(bbox.min, Coord { x: -4.0, y: 0.0 });
        assert_eq!(bbox.max, Coord { x: 1.0, y: 5.0 });
    }

    #[test]
    fn test_bbox_intersects() {
        let mut bbox1 = BBox::default();
        bbox1.extend(Coord { x: 0.0, y: 0.0 });
        bbox1.extend(Coord { x: 2.0, y: 2.0 });

        let mut bbox2 = BBox::default();
        bbox2.extend(Coord { x: 1.0, y: 1.0 });
        bbox2.extend(Coord { x: 3.0, y: 3.0 });

        assert!(bbox1.intersects(&bbox2));

        let mut bbox3 = BBox::default();
        bbox3.extend(Coord { x: 3.0, y: 3.0 });
        bbox3.extend(Coord { x: 4.0, y: 4.0 });

        assert!(!bbox1.intersects(&bbox3));
    }
}
