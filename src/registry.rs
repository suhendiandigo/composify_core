use pyo3::{
    prelude::*,
    types::{PyTuple, PyType},
};
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::{
    metadata::{MetadataSet, Qualifiers},
    rules::Rule,
    solve_parameters::SolveSpecificity,
    type_info::TypeInfo,
};

pub type TypeHash = isize;

#[derive(Default, Debug, Clone)]
pub struct TypeRegistry {
    subclasses: HashMap<TypeHash, HashSet<TypeHash>>,
    superclasses: HashMap<TypeHash, Vec<TypeHash>>,
}

/// Includes self type and builtin types.
fn resolve_bases<'a>(typing: &'a Bound<PyType>) -> PyResult<Vec<Bound<'a, PyType>>> {
    let superclasses = typing.mro();
    let mut result = Vec::new();
    for s in superclasses.iter() {
        let s = s.downcast_into::<PyType>()?;
        result.push(s);
    }
    Ok(result)
}

impl TypeRegistry {
    fn add_subclass(&mut self, superclass: TypeHash, subclass: TypeHash) {
        if let Some(subclasses) = self.subclasses.get_mut(&superclass) {
            subclasses.insert(subclass);
        } else {
            let mut subclasses = HashSet::new();
            subclasses.insert(subclass);
            self.subclasses.insert(superclass, subclasses);
        }
    }

    pub fn add(&mut self, typing: &Bound<PyType>) -> PyResult<()> {
        let type_hash = typing.hash()?;
        let mut superclass_hashes = Vec::new();
        for s in resolve_bases(typing)? {
            let superclass_hash = s.hash()?;
            superclass_hashes.push(superclass_hash);
            self.add_subclass(superclass_hash, type_hash);
        }
        self.superclasses.insert(type_hash, superclass_hashes);
        Ok(())
    }

    pub fn get_superclasses(&self, key: TypeHash) -> Option<&Vec<TypeHash>> {
        self.superclasses.get(&key)
    }

    pub fn get_subclasses(&self, key: TypeHash) -> Option<&HashSet<TypeHash>> {
        self.subclasses.get(&key)
    }
}

#[pyclass(module = "composify.core.registry")]
#[derive(Default)]
pub struct RuleRegistry {
    rules: HashMap<isize, BinaryHeap<Rule>>,
    types: TypeRegistry,
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

    pub fn inner_get(
        &self,
        py: Python,
        key: &TypeHash,
        attributes: &MetadataSet,
        qualifiers: &Qualifiers,
    ) -> PyResult<Option<Vec<&Rule>>> {
        let elements = if let Some(elements) = self.rules.get(key) {
            elements
        } else {
            return Ok(None);
        };
        // TODO: Need to check type Specificity.
        let mut rules: Vec<&Rule> = elements
            .iter()
            .filter(|r| attributes.issubset(&r.output_type.attributes))
            .collect();
        if !qualifiers.is_empty() {
            let mut qualified_rules = Vec::new();
            for e in rules.into_iter() {
                if qualifiers.qualify(py, &e.output_type.attributes)? {
                    qualified_rules.push(e);
                }
            }
            rules = qualified_rules;
        }
        Ok(Some(rules))
    }

    /// Get all superclasses including self type.
    pub fn get_super(&self, py: Python, type_info: &TypeInfo) -> PyResult<Option<Vec<&Rule>>> {
        if let Some(keys) = self.types.get_superclasses(type_info.type_hash) {
            let mut rules: Vec<&Rule> = Vec::new();
            for key in keys {
                if let Some(super_rules) =
                    self.inner_get(py, key, &type_info.attributes, &type_info.qualifiers)?
                {
                    rules.extend(super_rules);
                }
            }
            if rules.is_empty() {
                Ok(None)
            } else {
                Ok(Some(rules))
            }
        } else {
            Ok(None)
        }
    }

    /// Get all subclasses including self type.
    pub fn get_sub(&self, py: Python, type_info: &TypeInfo) -> PyResult<Option<Vec<&Rule>>> {
        if let Some(keys) = self.types.get_subclasses(type_info.type_hash) {
            let mut rules: Vec<&Rule> = Vec::new();
            for key in keys {
                if let Some(super_rules) =
                    self.inner_get(py, key, &type_info.attributes, &type_info.qualifiers)?
                {
                    rules.extend(super_rules);
                }
            }
            if rules.is_empty() {
                Ok(None)
            } else {
                Ok(Some(rules))
            }
        } else {
            Ok(None)
        }
    }

    /// Get exact type.
    pub fn get_exact(&self, py: Python, type_info: &TypeInfo) -> PyResult<Option<Vec<&Rule>>> {
        self.inner_get(
            py,
            &type_info.type_hash,
            &type_info.attributes,
            &type_info.qualifiers,
        )
    }

    /// Get using the specificity defined in the TypeInfo.
    pub fn get(&self, py: Python, type_info: &TypeInfo) -> PyResult<Option<Vec<&Rule>>> {
        match type_info.solve_parameter.specificity {
            SolveSpecificity::Exact => self.get_exact(py, type_info),
            SolveSpecificity::AllowSubclass => self.get_sub(py, type_info),
            SolveSpecificity::AllowSuperclass => self.get_super(py, type_info),
        }
    }

    pub fn clone_ref(&self, py: Python) -> Self {
        let mut map = HashMap::new();
        for (key, value) in self.rules.iter() {
            map.insert(*key, value.iter().map(|r| r.clone_ref(py)).collect());
        }
        Self {
            rules: map,
            types: self.types.clone(),
        }
    }
}

#[pymethods]
impl RuleRegistry {
    #[new]
    fn __new__() -> RuleRegistry {
        RuleRegistry::default()
    }

    pub fn add_rule(&mut self, rule: &Bound<Rule>) -> PyResult<()> {
        // let rule = rule.downcast::<Rule>()?;
        let py = rule.py();
        self.types
            .add(rule.borrow().output_type.inner_type.bind(py))?;
        self.add(rule.get().clone_ref(py));
        Ok(())
    }

    pub fn add_rules(&mut self, rules: &Bound<PyAny>) -> PyResult<()> {
        // let rule = rule.downcast::<Rule>()?;
        for rule in rules.iter()? {
            let rule = rule?;
            self.add_rule(rule.downcast::<Rule>()?)?;
        }
        Ok(())
    }
    pub fn get_rules<'py>(
        &mut self,
        type_info: Bound<'py, PyAny>,
    ) -> PyResult<Option<Bound<'py, PyTuple>>> {
        let py = type_info.py();
        let key = TypeInfo::parse(type_info)?;
        let rules = match self.get(py, &key)? {
            Some(e) => e,
            None => return Ok(None),
        };
        Ok(Some(PyTuple::new_bound(py, rules)))
    }
}
