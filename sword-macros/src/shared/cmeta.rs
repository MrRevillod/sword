use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Global CMeta stack for passing context between macro expansions
static CMETA_STACK: Lazy<Mutex<Option<CMetaStack>>> = Lazy::new(|| Mutex::new(None));

/// A simple stack-based context system for passing information between macros.
///
/// This is inspired by nidrs' CMeta system but simplified for Sword's needs.
/// It allows #[controller] to push context that #[get]/[post] can read.
#[derive(Debug, Clone)]
pub struct CMetaStack {
    data: HashMap<String, String>,
    parent: Option<Box<CMetaStack>>,
}

impl CMetaStack {
    /// Create a new empty CMeta level
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            parent: None,
        }
    }

    /// Push a new context level with the given key-value pair
    ///
    /// This will create a new level on the stack and set the previous
    /// level as its parent, allowing hierarchical lookups.
    pub fn push(key: &str, value: &str) {
        let mut stack = CMETA_STACK.lock().unwrap();

        let mut new_level = CMetaStack::new();
        new_level.data.insert(key.to_string(), value.to_string());

        if let Some(current) = stack.take() {
            new_level.parent = Some(Box::new(current));
        }

        *stack = Some(new_level);
    }

    /// Get a value from the stack by key
    ///
    /// This will search the current level and all parent levels
    /// until a value is found or the stack is exhausted.
    pub fn get(key: &str) -> Option<String> {
        let stack = CMETA_STACK.lock().unwrap();

        if let Some(current) = stack.as_ref() {
            current.get_recursive(key)
        } else {
            None
        }
    }

    /// Recursive helper for getting values from the stack
    fn get_recursive(&self, key: &str) -> Option<String> {
        // Check current level first
        if let Some(value) = self.data.get(key) {
            return Some(value.clone());
        }

        // Check parent levels
        if let Some(parent) = &self.parent {
            parent.get_recursive(key)
        } else {
            None
        }
    }
}
