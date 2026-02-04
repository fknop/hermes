use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Skill(String);

impl Skill {
    pub fn new(skill: String) -> Self {
        Skill(skill)
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}
