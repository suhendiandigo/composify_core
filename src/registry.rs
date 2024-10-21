use pyo3::{prelude::*, types::PyTuple};
use std::collections::HashMap;

use crate::{rules::Rule, type_info::TypeInfo};

#[pyclass(module = "composify.registry")]
#[derive(Default)]
pub struct RuleRegistry {
    rules: HashMap<isize, Vec<Rule>>,
}

impl RuleRegistry {
    pub fn add(&mut self, rule: Rule) {
        let key = rule.output_type.type_hash;
        let rules = match self.rules.get_mut(&key) {
            Some(r) => r,
            None => {
                self.rules.insert(key, Vec::new());
                self.rules.get_mut(&key).unwrap()
            }
        };
        rules.push(rule)
    }

    pub fn get(&self, type_info: &TypeInfo) -> Option<&Vec<Rule>> {
        let key = type_info.type_hash;
        self.rules.get(&key)
    }
}

#[pymethods]
impl RuleRegistry {
    #[new]
    fn __new__() -> RuleRegistry {
        RuleRegistry::default()
    }

    pub fn add_rule(&mut self, rule: Bound<'_, Rule>) -> PyResult<()> {
        // let rule = rule.downcast::<Rule>()?;
        self.add(rule.get().clone_ref(rule.py()));
        Ok(())
    }

    pub fn get_rules<'py>(
        &mut self,
        type_info: Bound<'py, PyAny>,
    ) -> PyResult<Option<Bound<'py, PyTuple>>> {
        // let rule = rule.downcast::<Rule>()?;
        let py = type_info.py();
        let key = TypeInfo::parse(type_info)?;
        let elements = match self.get(&key) {
            Some(e) => e,
            None => return Ok(None),
        };
        if key.qualifiers.is_empty() {
            Ok(Some(PyTuple::new_bound(py, elements)))
        } else {
            let mut qualified_rules: Vec<&Rule> = Vec::new();
            for e in elements {
                let qualified = key.qualifiers.qualify(py, &e.output_type.attributes)?;
                println!("{} {}", e, qualified);
                if qualified {
                    qualified_rules.push(e);
                }
            }
            Ok(Some(PyTuple::new_bound(py, qualified_rules)))
        }
    }
}
