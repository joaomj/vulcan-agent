use std::collections::HashMap;
use crate::error::ToolError;
use crate::types::ToolDescriptor;

pub struct ToolRegistry {
    tools: HashMap<String, ToolDescriptor>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, descriptor: ToolDescriptor) -> Result<(), ToolError> {
        let name = descriptor.name.clone();
        if self.tools.contains_key(&name) {
            return Err(ToolError::Conflict(format!("Tool '{}' already registered", name)));
        }
        self.tools.insert(name, descriptor);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&ToolDescriptor> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<&ToolDescriptor> {
        self.tools.values().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ApprovalClass, IsolationLevel, CancellationSupport};

    fn make_descriptor(name: &str) -> ToolDescriptor {
        ToolDescriptor {
            name: name.to_string(),
            description: String::new(),
            input_schema: serde_json::Value::Object(Default::default()),
            approval_class: ApprovalClass::Never,
            isolation_level: IsolationLevel::None,
            cancellation_support: CancellationSupport::None,
        }
    }

    #[test]
    fn test_register_and_list() {
        let mut reg = ToolRegistry::new();
        reg.register(make_descriptor("read")).unwrap();
        reg.register(make_descriptor("write")).unwrap();
        assert_eq!(reg.list().len(), 2);
    }

    #[test]
    fn test_register_collision() {
        let mut reg = ToolRegistry::new();
        reg.register(make_descriptor("read")).unwrap();
        let err = reg.register(make_descriptor("read")).unwrap_err();
        assert!(matches!(err, ToolError::Conflict(_)));
    }

    #[test]
    fn test_get() {
        let mut reg = ToolRegistry::new();
        reg.register(make_descriptor("read")).unwrap();
        assert!(reg.get("read").is_some());
        assert!(reg.get("nonexistent").is_none());
    }
}
