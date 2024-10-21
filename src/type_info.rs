use pyo3::prelude::*;
use pyo3::types::{PyBool, PyTuple, PyType};
use pyo3::{intern, types::PySequence};
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::metadata::MetadataSet;

#[pyclass]
#[derive(Debug, Default)]
pub struct Qualifiers(pub Vec<Py<PyAny>>);

impl Qualifiers {
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Qualifiers(self.0.iter().map(|a| a.clone_ref(py)).collect())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn qualify(&self, py: Python<'_>, attrs: &MetadataSet) -> PyResult<bool> {
        let args = PyTuple::new_bound(py, [attrs.clone_ref(py).into_py(py)]);
        for q in self.0.iter() {
            // let q = q.bind(py).call_method1(intern!(py, "qualify"), &args)?;
            let q = q.bind(py).call1(&args)?;
            let q = q.downcast_into::<PyBool>()?;
            if !q.is_true() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl ToPyObject for Qualifiers {
    fn to_object(&self, py: Python<'_>) -> pyo3::Py<PyAny> {
        self.clone_ref(py).into_py(py)
    }
}

fn split_metadata(metadata: &Bound<'_, PySequence>) -> PyResult<(MetadataSet, Qualifiers)> {
    let py = metadata.py();
    let mut attributes = Vec::new();
    let mut qualifiers = Vec::new();
    for py_element in metadata.iter()?.flatten() {
        match py_element.getattr(intern!(py, "qualify")) {
            Ok(f) => {
                qualifiers.push(f.unbind());
            }
            Err(..) => {
                attributes.push(py_element);
            }
        }
    }
    Ok((MetadataSet::new(attributes)?, Qualifiers(qualifiers)))
}

#[pyclass(get_all, frozen)]
#[derive(Debug)]
pub struct TypeInfo {
    pub type_name: String,
    pub type_module: String,
    pub type_hash: isize,
    pub inner_type: Py<PyType>,
    pub attributes: MetadataSet,
    pub qualifiers: Qualifiers,
}

impl Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TypeInfo({}.{}, attrs={:?}, qualifiers={})",
            self.type_module,
            self.type_name,
            self.attributes,
            self.qualifiers.0.len()
        )
    }
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
        let (attributes, qualifiers) = match metadata {
            Some(metadata) => split_metadata(&metadata)?,
            None => (MetadataSet::default(), Qualifiers::default()),
        };
        Ok(TypeInfo {
            type_name: t.name()?.to_string(),
            type_module: t.module()?.to_string(),
            type_hash: t.hash()?,
            inner_type: t.clone().unbind(),
            attributes,
            qualifiers,
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

    fn __repr__(&self) -> String {
        format!(
            "TypeInfo({}.{}, attrs={:?}, qualifiers={})",
            self.type_module,
            self.type_name,
            self.attributes,
            self.qualifiers.0.len()
        )
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
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_hash == other.type_hash
    }
}
