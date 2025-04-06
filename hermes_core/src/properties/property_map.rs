use crate::{edge_direction::EdgeDirection, properties::property::Property};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Default)]
struct SmallMap<T>(Vec<(Property, T)>);
impl<T> SmallMap<T> {
    fn new() -> Self {
        SmallMap(Vec::new())
    }
    fn get(&self, property: &Property) -> Option<&T> {
        self.0.iter().find(|(p, _)| p == property).map(|(_, v)| v)
    }

    fn insert(&mut self, property: Property, value: T) {
        self.0.push((property, value));
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Default)]
struct DirectionalMap<T> {
    forward: SmallMap<T>,
    backward: SmallMap<T>,
}

impl<T: Copy> DirectionalMap<T> {
    fn new() -> Self {
        DirectionalMap {
            forward: SmallMap::new(),
            backward: SmallMap::new(),
        }
    }

    fn get(&self, property: Property, direction: EdgeDirection) -> Option<&T> {
        match direction {
            EdgeDirection::Forward => self.forward.get(&property),
            EdgeDirection::Backward => self.backward.get(&property),
        }
    }

    pub fn insert(&mut self, property: Property, direction: EdgeDirection, value: T) -> Option<T> {
        match direction {
            EdgeDirection::Forward => self.forward.insert(property, value),
            EdgeDirection::Backward => self.backward.insert(property, value),
        }

        Some(value)
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, Default)]
pub struct EdgePropertyMap {
    f32_values: DirectionalMap<f32>,
    bool_values: DirectionalMap<bool>,
    u8_values: DirectionalMap<u8>,
    usize_values: SmallMap<usize>,
}

macro_rules! define_directional_access_functions {
    ($type:ty, $field:ident) => {
        paste::paste! {
            pub fn [<get_ $type>](&self, property: Property, direction: EdgeDirection) -> Option<$type> {
                self.$field.get(property, direction).cloned()
            }

            pub fn [<insert_ $type>](&mut self, property: Property, direction: EdgeDirection, value: $type) {
                self.$field.insert(property, direction, value);
            }
        }
    };
}

impl EdgePropertyMap {
    pub fn get_usize(&self, property: Property) -> Option<usize> {
        self.usize_values.get(&property).cloned()
    }

    pub fn insert_usize(&mut self, property: Property, value: usize) {
        self.usize_values.insert(property, value);
    }

    define_directional_access_functions!(f32, f32_values);
    define_directional_access_functions!(u8, u8_values);
    define_directional_access_functions!(bool, bool_values);
}
