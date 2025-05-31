use alloc::{collections::BTreeMap, string::String, vec::Vec};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::intereperter::components::tokens::Tokens;

#[derive(Clone, Default)]
pub struct Environment {
    pub variables: BTreeMap<String, f64>,
}

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<String>,
    pub body_tokens: Vec<Tokens>,
    pub body_start: usize,
    pub body_end: usize,
}

impl Environment {
    pub fn new() -> Self {
        Environment::default()
    }

    pub fn set(&mut self, name: String, value: f64) {
        self.variables.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        self.variables.get(name).copied()
    }
}

lazy_static! {
    pub static ref GLOBAL_ENV: Mutex<Environment> = Mutex::new(Environment::new());
    pub static ref FUNCTION_REGISTRY: Mutex<BTreeMap<String, Function>> =
        Mutex::new(BTreeMap::new());
}
