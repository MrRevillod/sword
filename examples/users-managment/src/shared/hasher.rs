use bcrypt::hash;
use serde::Deserialize;
use sword::prelude::*;

#[derive(Clone, Deserialize)]
#[config(key = "hasher")]
pub struct HasherConfig {
    pub cost: u32,
}

#[injectable(provider)]
pub struct Hasher {
    cost: u32,
}

impl Hasher {
    pub fn new(config: &HasherConfig) -> Self {
        Self { cost: config.cost }
    }

    pub fn hash(&self, password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, self.cost)
    }
}
