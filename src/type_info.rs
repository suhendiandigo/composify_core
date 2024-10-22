use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{intern, types::PySequence};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::metadata::{AttributeQualifier, MetadataSet, Qualifiers};
use crate::solve_parameters::{SolveCardinality, SolveParameter, SolveSpecificity};

fn parse_metadata(
    metadata: &Bound<'_, PySequence>,
) -> PyResult<(MetadataSet, Qualifiers, SolveParameter)> {
    let py = metadata.py();
    let mut attributes = Vec::new();
    let mut qualifiers = Vec::new();
    let mut solve_parameter = SolveParameter::default();
    for py_element in metadata.iter()?.flatten() {
        match py_element.getattr(intern!(py, "qualify")) {
            Ok(f) => {
                qualifiers.push(f);
            }
            Err(..) => {
                if let Ok(c) = py_element.downcast::<SolveCardinality>() {
                    let c = c.get();
                    solve_parameter.cardinality = c.clone();
                } else if let Ok(s) = py_element.downcast::<SolveSpecificity>() {
                    let s = s.get();
                    solve_parameter.specificity = s.clone();
                } else {
                    attributes.push(py_element);
                }
            }
        }
    }
    let metadata = MetadataSet::new(attributes)?;
    if !metadata.is_empty() {
        qualifiers.push(
            AttributeQualifier(Py::new(py, metadata.clone_ref(py))?)
                .to_object(py)
                .into_bound(py),
        );
    }
    Ok((metadata, Qualifiers::__new__(qualifiers)?, solve_parameter))
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

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "TypeInfo({}.{}, attrs={}, qualifiers={}, solve={})",
            self.type_module,
            self.type_name,
            self.attributes.__repr__(py)?,
            self.qualifiers.__repr__(py)?,
            self.solve_parameter,
        ))
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
