use pyo3::{
    prelude::*,
    types::{PySequence, PyTuple},
};
use std::collections::{BinaryHeap, HashMap};

use crate::{rules::Rule, type_info::TypeInfo};

#[pyclass(module = "composify.core.registry")]
#[derive(Default)]
pub struct RuleRegistry {
    rules: HashMap<isize, BinaryHeap<Rule>>,
}

impl RuleRegistry {
    pub fn add(&mut self, rule: Rule) {
        let key = rule.output_type.type_hash;
        let rules = match self.rules.get_mut(&key) {
            Some(r) => r,
            None => {
                self.rules.insert(key, BinaryHeap::new());
                self.rules.get_mut(&key).unwrap()
            }
        };
        rules.push(rule)
    }

    pub fn get(&self, type_info: &TypeInfo) -> Option<&BinaryHeap<Rule>> {
        let key = type_info.type_hash;
        self.rules.get(&key)
    }

    pub fn clone_ref(&self, py: Python) -> Self {
        let mut map = HashMap::new();
        for (key, value) in self.rules.iter() {
            map.insert(*key, value.iter().map(|r| r.clone_ref(py)).collect());
        }
        Self { rules: map }
    }
}

#[pymethods]
impl RuleRegistry {
    #[new]
    fn __new__() -> RuleRegistry {
        RuleRegistry::default()
    }

    pub fn add_rule(&mut self, rule: Bound<Rule>) -> PyResult<()> {
        // let rule = rule.downcast::<Rule>()?;
        self.add(rule.get().clone_ref(rule.py()));
        Ok(())
    }

    pub fn add_rules(&mut self, rules: Bound<PySequence>) -> PyResult<()> {
        // let rule = rule.downcast::<Rule>()?;
        for rule in rules.iter()? {
            let rule = rule?;
            self.add(rule.downcast::<Rule>()?.get().clone_ref(rule.py()));
        }
        Ok(())
    }
    pub fn get_rules<'py>(
        &mut self,
        type_info: Bound<'py, PyAny>,
    ) -> PyResult<Option<Bound<'py, PyTuple>>> {
        let py = type_info.py();
        let key = TypeInfo::parse(type_info)?;
        let elements = match self.get(&key) {
            Some(e) => e,
            None => return Ok(None),
        };
        // TODO: Need to check type Specificity.
        if key.qualifiers.is_empty() {
            Ok(Some(PyTuple::new_bound(py, elements)))
        } else {
            let mut qualified_rules: Vec<&Rule> = Vec::new();
            for e in elements {
                let attrs = &e.output_type.attributes;
                let qualified =
                    key.attributes.issubset(attrs) && key.qualifiers.qualify(py, attrs)?;
                if qualified {
                    qualified_rules.push(e);
                }
            }
            Ok(Some(PyTuple::new_bound(py, qualified_rules)))
        }
    }
}
