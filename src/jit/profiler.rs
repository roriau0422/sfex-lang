// Profiler for detecting hot code paths

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// JIT thresahold
const JIT_THRESHOLD: usize = 100;

#[derive(Debug, Clone)]
pub struct Profiler {
    call_counts: Arc<RwLock<HashMap<(String, String), usize>>>,
    jit_compiled: Arc<RwLock<HashMap<(String, String), bool>>>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            call_counts: Arc::new(RwLock::new(HashMap::new())),
            jit_compiled: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn record_call(&self, concept: &str, method: &str) {
        let key = (concept.to_string(), method.to_string());
        let mut counts = self.call_counts.write().expect("lock poisoned");
        *counts.entry(key).or_insert(0) += 1;
    }

    pub fn should_jit(&self, concept: &str, method: &str) -> bool {
        let key = (concept.to_string(), method.to_string());

        {
            let compiled = self.jit_compiled.read().expect("lock poisoned");
            if compiled.get(&key).copied().unwrap_or(false) {
                return false;
            }
        }

        let counts = self.call_counts.read().expect("lock poisoned");
        counts.get(&key).copied().unwrap_or(0) >= JIT_THRESHOLD
    }

    pub fn mark_compiled(&self, concept: &str, method: &str) {
        let key = (concept.to_string(), method.to_string());
        let mut compiled = self.jit_compiled.write().expect("lock poisoned");
        compiled.insert(key, true);
    }

    pub fn get_call_count(&self, concept: &str, method: &str) -> usize {
        let key = (concept.to_string(), method.to_string());
        let counts = self.call_counts.read().expect("lock poisoned");
        counts.get(&key).copied().unwrap_or(0)
    }

    pub fn get_hot_functions(&self) -> Vec<(String, String, usize)> {
        let counts = self.call_counts.read().expect("lock poisoned");
        let mut hot: Vec<_> = counts
            .iter()
            .filter(|&(_, count)| *count >= JIT_THRESHOLD)
            .map(|((c, m), count)| (c.clone(), m.clone(), *count))
            .collect();
        hot.sort_by(|a, b| b.2.cmp(&a.2)); // Sort by count descending
        hot
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}
