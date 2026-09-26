use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::crd::MutationRuleSpec;

#[derive(Clone, Debug)]
pub struct CompiledRule {
    pub name: String,
    pub namespace: Option<String>,
    pub priority: i32,
    pub spec: MutationRuleSpec,
}

#[derive(Clone, Default)]
pub struct RuleStore {
    rules: Arc<RwLock<HashMap<String, CompiledRule>>>,
}

impl RuleStore {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert or update a rule
    pub async fn upsert(&self, key: String, rule: CompiledRule) {
        let mut map = self.rules.write().await;
        map.insert(key, rule);
    }

    /// Remove a rule by key
    pub async fn remove(&self, key: &str) {
        let mut map = self.rules.write().await;
        map.remove(key);
    }

    /// Get all rules sorted by priority descending (highest priority executed first)
    pub async fn get_all_sorted(&self) -> Vec<CompiledRule> {
        let map = self.rules.read().await;
        let mut rules: Vec<CompiledRule> = map.values().cloned().collect();
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        rules
    }

    /// Get total active rule count
    pub async fn count(&self) -> usize {
        let map = self.rules.read().await;
        map.len()
    }
}
