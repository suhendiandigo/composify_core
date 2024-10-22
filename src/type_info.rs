use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{intern, types::PySequence};
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::metadata::{MetadataSet, Qualifiers, QUALIFY_METHOD_NAME};
use crate::solve_parameters::{SolveCardinality, SolveParameter, SolveSpecificity};

fn parse_metadata(
    metadata: &Bound<'_, PySequence>,
) -> PyResult<(MetadataSet, Qualifiers, SolveParameter)> {
    let py = metadata.py();
    let mut attributes = Vec::new();
    let mut qualifiers = Vec::new();
    let mut solve_parameter = SolveParameter::default();
    for py_element in metadata.iter()?.flatten() {
        if py_element.hasattr(intern!(py, QUALIFY_METHOD_NAME))? {
            qualifiers.push(py_element);
        } else if let Ok(c) = py_element.downcast::<SolveCardinality>() {
            let c = c.get();
            solve_parameter.cardinality = c.clone();
        } else if let Ok(s) = py_element.downcast::<SolveSpecificity>() {
            let s = s.get();
            solve_parameter.specificity = s.clone();
        } else {
            attributes.push(py_element);
        }
    }
    Ok((
        MetadataSet::new(attributes)?,
        Qualifiers::__new__(qualifiers)?,
        solve_parameter,
    ))
}

#[pyclass(get_all, frozen, module = "composify")]
#[derive(Debug)]
pub struct TypeInfo {
    pub type_name: String,
    pub type_module: String,
    pub type_hash: isize,
    pub inner_type: Py<PyType>,
    pub attributes: MetadataSet,
    pub qualifiers: Qualifiers,
    pub solve_parameter: SolveParameter,
}

#[pymethods]
impl TypeInfo {
    #[new]
    #[pyo3(signature = (type_annotation, metadata=None))]
    pub fn __new__(
        type_annotation: &Bound<'_, PyType>,
        metadata: Option<Bound<'_, PySequence>>,
    ) -> PyResult<TypeInfo> {
        let t = type_annotation.downcast::<PyType>()?;
        let (attributes, qualifiers, solve_parameter) = match metadata {
            Some(metadata) => parse_metadata(&metadata)?,
            None => (
                MetadataSet::default(),
                Qualifiers::default(),
                SolveParameter::default(),
            ),
        };
        Ok(TypeInfo {
            type_name: t.name()?.to_string(),
            type_module: t.module()?.to_string(),
            type_hash: t.hash()?,
            inner_type: t.clone().unbind(),
            attributes,
            qualifiers,
            solve_parameter,
        })
    }

    #[staticmethod]
    pub fn parse(type_annotation: Bound<'_, PyAny>) -> PyResult<TypeInfo> {
        let py = type_annotation.py();
        let t = match type_annotation.downcast::<PyType>() {
            Ok(t) => t,
            Err(..) => {
                if type_annotation.hasattr(intern!(py, "__origin__"))? {
                    let origin = type_annotation.getattr(intern!(py, "__origin__"))?;
                    &origin.downcast_into::<PyType>()?
                } else {
                    let a = type_annotation.downcast_into::<TypeInfo>()?;
                    return Ok(a.get().clone_ref(py));
                }
            }
        };
        let metadata = if type_annotation.hasattr(intern!(py, "__metadata__"))? {
            Some(
                type_annotation
                    .getattr(intern!(py, "__metadata__"))?
                    .downcast_into::<PySequence>()?,
            )
        } else {
            None
        };
        TypeInfo::__new__(t, metadata)
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(self.to_type_string())
    }

    fn __hash__(&self) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        Ok(hasher.finish())
    }
}

impl TypeInfo {
    pub fn clone_ref(&self, py: Python<'_>) -> TypeInfo {
        TypeInfo {
            type_name: self.type_name.clone(),
            type_module: self.type_module.clone(),
            type_hash: self.type_hash,
            inner_type: self.inner_type.clone_ref(py),
            attributes: self.attributes.clone_ref(py),
            qualifiers: self.qualifiers.clone_ref(py),
            solve_parameter: self.solve_parameter.clone(),
        }
    }

    #[inline(always)]
    pub fn canonical_name(&self) -> String {
        if self.type_module == "builtins" {
            self.type_name.clone()
        } else {
            format!("{}.{}", self.type_module, self.type_name)
        }
    }

    pub fn to_type_string(&self) -> String {
        let mut annotations: Vec<String> = Vec::new();
        if !self.attributes.is_empty() {
            for attr in self.attributes.iter() {
                annotations.push(attr.to_string());
            }
        }
        if !self.qualifiers.is_empty() {
            for qualifier in self.qualifiers.iter() {
                annotations.push(qualifier.to_string());
            }
        }
        if annotations.is_empty() {
            format!(
                "{}({}{})",
                self.canonical_name(),
                self.solve_parameter.specificity.symbol(),
                self.solve_parameter.cardinality.symbol()
            )
        } else {
            format!(
                "{}({}{}, {})",
                self.canonical_name(),
                self.solve_parameter.specificity.symbol(),
                self.solve_parameter.cardinality.symbol(),
                annotations.join(", ")
            )
        }
    }
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TypeInfo({}, attrs={}, qualifiers={}, solve={})",
            self.inner_type, self.attributes, self.qualifiers, self.solve_parameter,
        )
    }
}

impl ToPyObject for TypeInfo {
    fn to_object(&self, py: Python<'_>) -> pyo3::Py<PyAny> {
        self.clone_ref(py).into_py(py)
    }
}

impl Hash for TypeInfo {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.type_hash.hash(state);
        self.attributes.hash(state);
        self.qualifiers.hash(state);
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_hash == other.type_hash
            && self.attributes == other.attributes
            && self.qualifiers == other.qualifiers
    }
}

impl Eq for TypeInfo {}
