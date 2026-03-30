use std::collections::HashMap;
use std::sync::LazyLock as Lazy;
use std::sync::Mutex;

static EXPANSION_CONTEXT_STACK: Lazy<Mutex<Option<ExpansionContextStack>>> =
    Lazy::new(|| Mutex::new(None));

/// A simple stack-based context system for passing information between macros.
#[derive(Debug, Clone)]
pub struct ExpansionContextStack {
    data: HashMap<String, String>,
    parent: Option<Box<ExpansionContextStack>>,
}

pub struct WebControllerContext<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub has_sword_interceptors: bool,
}

#[cfg(feature = "socketio-controllers")]
pub struct SocketIoControllerContext<'a> {
    pub name: &'a str,
    pub namespace: &'a str,
}

impl ExpansionContextStack {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            parent: None,
        }
    }

    pub fn push(key: &str, value: &str) {
        let mut stack = EXPANSION_CONTEXT_STACK.lock().unwrap();
        let mut new_level = Self::new();

        new_level.data.insert(key.to_string(), value.to_string());

        if let Some(current) = stack.take() {
            new_level.parent = Some(Box::new(current));
        }

        *stack = Some(new_level);
    }

    pub fn get(key: &str) -> Option<String> {
        let stack = EXPANSION_CONTEXT_STACK.lock().unwrap();

        if let Some(current) = stack.as_ref() {
            current.get_recursive(key)
        } else {
            None
        }
    }

    fn get_recursive(&self, key: &str) -> Option<String> {
        if let Some(value) = self.data.get(key) {
            return Some(value.clone());
        }

        self.parent
            .as_ref()
            .and_then(|parent| parent.get_recursive(key))
    }
}

pub type CMetaStack = ExpansionContextStack;

pub fn push_web_controller_context(ctx: WebControllerContext<'_>) {
    ExpansionContextStack::push("controller_name", ctx.name);
    ExpansionContextStack::push("controller_path", ctx.path);
    ExpansionContextStack::push("controller_kind", "web");
    ExpansionContextStack::push(
        "controller_has_sword_interceptors",
        if ctx.has_sword_interceptors {
            "true"
        } else {
            "false"
        },
    );
}

#[cfg(feature = "socketio-controllers")]
pub fn push_socketio_controller_context(ctx: SocketIoControllerContext<'_>) {
    ExpansionContextStack::push("controller_name", ctx.name);
    ExpansionContextStack::push("controller_path", ctx.namespace);
    ExpansionContextStack::push("controller_kind", "socketio");
    ExpansionContextStack::push("socketio_controller_name", ctx.name);
    ExpansionContextStack::push("socketio_namespace", ctx.namespace);
}
