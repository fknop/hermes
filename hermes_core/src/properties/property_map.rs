use crate::properties::property::Property;
use std::collections::HashMap;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug)]
pub struct EdgePropertyMap {
    backward_bool_values: HashMap<String, bool>,
    forward_bool_values: HashMap<String, bool>,
    forward_u8_values: HashMap<String, u8>,
    backward_u8_values: HashMap<String, u8>,
    usize_values: HashMap<String, usize>,
}

pub type EdgeDirection = bool;
pub const FORWARD_EDGE: EdgeDirection = true;
pub const BACKWARD_EDGE: EdgeDirection = false;

impl EdgePropertyMap {
    pub fn new() -> EdgePropertyMap {
        EdgePropertyMap {
            forward_u8_values: HashMap::new(),
            backward_u8_values: HashMap::new(),
            forward_bool_values: HashMap::new(),
            backward_bool_values: HashMap::new(),
            usize_values: HashMap::new(),
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
            FORWARD_EDGE => self.forward_u8_values.get(&property.as_string()).cloned(),
            BACKWARD_EDGE => self.backward_u8_values.get(&property.as_string()).cloned(),
        }
    }

    pub fn get_bool(&self, property: Property, direction: EdgeDirection) -> Option<bool> {
        match direction {
            FORWARD_EDGE => self.forward_bool_values.get(&property.as_string()).cloned(),
            BACKWARD_EDGE => self
                .backward_bool_values
                .get(&property.as_string())
                .cloned(),
        }
    }

    pub fn get_usize(&self, property: Property) -> Option<usize> {
        self.usize_values.get(&property.as_string()).cloned()
    }

    pub fn insert_u8(
        &mut self,
        property: Property,
        direction: EdgeDirection,
        value: u8,
    ) -> Option<u8> {
        match direction {
            FORWARD_EDGE => self.forward_u8_values.insert(property.as_string(), value),
            BACKWARD_EDGE => self.backward_u8_values.insert(property.as_string(), value),
        }
    }
    pub fn insert_bool(
        &mut self,
        property: Property,
        direction: EdgeDirection,
        value: bool,
    ) -> Option<bool> {
        match direction {
            FORWARD_EDGE => self.forward_bool_values.insert(property.as_string(), value),
            BACKWARD_EDGE => self
                .backward_bool_values
                .insert(property.as_string(), value),
        }
    }

    pub fn insert_usize(&mut self, property: Property, value: usize) -> Option<usize> {
        self.usize_values.insert(property.as_string(), value)
    }
}

impl Default for EdgePropertyMap {
    fn default() -> Self {
        Self::new()
    }
}
