use crate::{edge_direction::EdgeDirection, properties::property::Property};

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug)]
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

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug)]
pub struct EdgePropertyMap {
    backward_bool_values: SmallMap<bool>,
    forward_bool_values: SmallMap<bool>,
    forward_u8_values: SmallMap<u8>,
    backward_u8_values: SmallMap<u8>,
    usize_values: SmallMap<usize>,
}

impl EdgePropertyMap {
    pub fn new() -> EdgePropertyMap {
        EdgePropertyMap {
            forward_u8_values: SmallMap::new(),
            backward_u8_values: SmallMap::new(),
            forward_bool_values: SmallMap::new(),
            backward_bool_values: SmallMap::new(),
            usize_values: SmallMap::new(),
        }
    }

    pub fn as_reversed(&self) -> EdgePropertyMap {
        EdgePropertyMap {
            forward_u8_values: self.backward_u8_values.clone(),
            backward_u8_values: self.forward_u8_values.clone(),
            forward_bool_values: self.backward_bool_values.clone(),
            backward_bool_values: self.forward_bool_values.clone(),
            usize_values: self.usize_values.clone(),
        }
    }

    pub fn get_u8(&self, property: Property, direction: EdgeDirection) -> Option<u8> {
        match direction {
            EdgeDirection::Forward => self.forward_u8_values.get(&property).cloned(),
            EdgeDirection::Backward => self.backward_u8_values.get(&property).cloned(),
        }
    }

    pub fn get_bool(&self, property: Property, direction: EdgeDirection) -> Option<bool> {
        match direction {
            EdgeDirection::Forward => self.forward_bool_values.get(&property).cloned(),
            EdgeDirection::Backward => self.backward_bool_values.get(&property).cloned(),
        }
    }

    pub fn get_usize(&self, property: Property) -> Option<usize> {
        self.usize_values.get(&property).cloned()
    }

    pub fn insert_u8(
        &mut self,
        property: Property,
        direction: EdgeDirection,
        value: u8,
    ) -> Option<u8> {
        match direction {
            EdgeDirection::Forward => self.forward_u8_values.insert(property, value),
            EdgeDirection::Backward => self.backward_u8_values.insert(property, value),
        }

        Some(value)
    }
    pub fn insert_bool(
        &mut self,
        property: Property,
        direction: EdgeDirection,
        value: bool,
    ) -> Option<bool> {
        match direction {
            EdgeDirection::Forward => self.forward_bool_values.insert(property, value),
            EdgeDirection::Backward => self.backward_bool_values.insert(property, value),
        }

        Some(value)
    }

    pub fn insert_usize(&mut self, property: Property, value: usize) -> Option<usize> {
        self.usize_values.insert(property, value);

        Some(value)
    }
}

impl Default for EdgePropertyMap {
    fn default() -> Self {
        Self::new()
    }
}
