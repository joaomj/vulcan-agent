use crate::error::SkillError;
use crate::types::SkillMeta;
use std::collections::HashMap;

pub struct SkillsLoader;

impl SkillsLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn discover(&self) -> Result<HashMap<String, SkillMeta>, SkillError> {
        Ok(HashMap::new())
    }
}

impl Default for SkillsLoader {
    fn default() -> Self {
        Self::new()
    }
}
