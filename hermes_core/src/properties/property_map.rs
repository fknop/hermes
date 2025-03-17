use crate::properties::property::Property;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct EdgePropertyMap {
    backward_bool_values: HashMap<Property, bool>,
    forward_bool_values: HashMap<Property, bool>,
    forward_u8_values: HashMap<Property, u8>,
    backward_u8_values: HashMap<Property, u8>,

    usize_values: HashMap<Property, usize>,
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

    pub fn get_u8(&self, property: Property, direction: EdgeDirection) -> Option<u8> {
        match direction {
            FORWARD_EDGE => self.forward_u8_values.get(&property).cloned(),
            BACKWARD_EDGE => self.backward_u8_values.get(&property).cloned(),
        }
    }

    pub fn get_bool(&self, property: Property, direction: EdgeDirection) -> Option<bool> {
        match direction {
            FORWARD_EDGE => self.forward_bool_values.get(&property).cloned(),
            BACKWARD_EDGE => self.backward_bool_values.get(&property).cloned(),
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
            FORWARD_EDGE => self.forward_u8_values.insert(property, u8::from(value)),
            BACKWARD_EDGE => self.backward_u8_values.insert(property, u8::from(value)),
        }
    }
    pub fn insert_bool(
        &mut self,
        property: Property,
        direction: EdgeDirection,
        value: bool,
    ) -> Option<bool> {
        match direction {
            FORWARD_EDGE => self.forward_bool_values.insert(property, value),
            BACKWARD_EDGE => self.backward_bool_values.insert(property, value),
        }
    }

    pub fn insert_usize(&mut self, property: Property, value: usize) -> Option<usize> {
        self.usize_values.insert(property, value)
    }
}
