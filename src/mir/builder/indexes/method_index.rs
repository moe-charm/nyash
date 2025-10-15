/*!
 * MethodIndexBox - Centralized method index management
 *
 * Manages three types of method indexes:
 * - Static methods: name -> [(BoxName, arity)]
 * - Instance methods: (BoxName, method, arity) set
 * - Tail index: ".method/arity" -> [full_names] (performance optimization)
 */

use crate::mir::MirFunction;
use std::collections::{HashMap, HashSet};

/// Method index registry for centralized tracking
#[derive(Debug, Clone, Default)]
pub struct MethodIndexBox {
    /// Static method index: name -> [(BoxName, arity)]
    static_methods: HashMap<String, Vec<(String, usize)>>,

    /// Instance method index: (BoxName, method, arity)
    instance_methods: HashSet<(String, String, usize)>,

    /// Fast lookup tail index: ".method/arity" -> [full_names]
    tail_index: HashMap<String, Vec<String>>,

    /// Source size snapshot to detect when to rebuild tail index
    tail_index_source_len: usize,
}

impl MethodIndexBox {
    /// Create a new method index
    pub fn new() -> Self {
        Self {
            static_methods: HashMap::new(),
            instance_methods: HashSet::new(),
            tail_index: HashMap::new(),
            tail_index_source_len: 0,
        }
    }

    /// Register a static method
    pub fn register_static_method(&mut self, name: String, box_name: String, arity: usize) {
        self.static_methods
            .entry(name)
            .or_insert_with(Vec::new)
            .push((box_name, arity));
    }

    /// Register an instance method
    pub fn register_instance_method(&mut self, box_name: String, method: String, arity: usize) {
        self.instance_methods.insert((box_name, method, arity));
    }

    /// Check if an instance method exists
    pub fn instance_method_exists(&self, box_name: &str, method: &str, arity: usize) -> bool {
        self.instance_methods
            .contains(&(box_name.to_string(), method.to_string(), arity))
    }

    /// Get static method index (read-only reference)
    pub fn static_methods(&self) -> &HashMap<String, Vec<(String, usize)>> {
        &self.static_methods
    }

    /// Rebuild tail index from function names
    fn rebuild_tail_index(&mut self, functions: &HashMap<String, MirFunction>) {
        self.tail_index.clear();
        for name in functions.keys() {
            if let (Some(dot), Some(slash)) = (name.rfind('.'), name.rfind('/')) {
                if slash > dot {
                    let tail = &name[dot..];
                    self.tail_index
                        .entry(tail.to_string())
                        .or_insert_with(Vec::new)
                        .push(name.clone());
                }
            }
        }
        self.tail_index_source_len = functions.len();
    }

    /// Ensure tail index is up-to-date (rebuilds if needed)
    fn ensure_tail_index(&mut self, functions: &HashMap<String, MirFunction>) {
        let need_rebuild = self.tail_index_source_len != functions.len();
        if need_rebuild {
            self.rebuild_tail_index(functions);
        }
    }

    /// Find method candidates by method name and arity
    pub fn find_candidates(
        &mut self,
        functions: &HashMap<String, MirFunction>,
        method: &str,
        arity: usize,
    ) -> Vec<String> {
        self.ensure_tail_index(functions);
        let tail = format!(".{}/{}", method, arity);
        self.tail_index.get(&tail).cloned().unwrap_or_default()
    }

    /// Find method candidates by tail string (e.g., ".str/0")
    pub fn find_candidates_by_tail(
        &mut self,
        functions: &HashMap<String, MirFunction>,
        tail: &str,
    ) -> Vec<String> {
        self.ensure_tail_index(functions);
        self.tail_index.get(tail).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_index_static_registration() {
        let mut index = MethodIndexBox::new();

        index.register_static_method("main".to_string(), "Main".to_string(), 0);
        index.register_static_method("main".to_string(), "Test".to_string(), 1);

        let static_methods = index.static_methods();
        assert_eq!(static_methods.get("main").unwrap().len(), 2);
    }

    #[test]
    fn test_method_index_instance_exists() {
        let mut index = MethodIndexBox::new();

        index.register_instance_method("Person".to_string(), "getName".to_string(), 0);

        assert!(index.instance_method_exists("Person", "getName", 0));
        assert!(!index.instance_method_exists("Person", "getName", 1));
        assert!(!index.instance_method_exists("Animal", "getName", 0));
    }

    #[test]
    fn test_method_index_tail_lookup() {
        let mut index = MethodIndexBox::new();
        let mut functions = HashMap::new();

        // Add mock functions
        functions.insert(
            "Person.getName/0".to_string(),
            MirFunction::new("Person.getName/0".to_string(), vec![], None),
        );
        functions.insert(
            "Animal.getName/0".to_string(),
            MirFunction::new("Animal.getName/0".to_string(), vec![], None),
        );

        let candidates = index.find_candidates(&functions, "getName", 0);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"Person.getName/0".to_string()));
        assert!(candidates.contains(&"Animal.getName/0".to_string()));
    }
}
