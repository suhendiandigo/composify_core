use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyTuple, PyType};
use pyo3::PyObject;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[pyclass(
    get_all,
    frozen,
    eq,
    hash,
    subclass,
    module = "composify.core.metadata"
)]
#[derive(Debug, Default)]
pub struct MetadataSet {
    map: HashMap<isize, PyObject>,
    hash: u64,
}

impl MetadataSet {
    pub fn new(items: Vec<Bound<'_, PyAny>>) -> PyResult<MetadataSet> {
        let mut hasher = DefaultHasher::default();
        let mut map = HashMap::new();
        for item in items {
            hasher.write_isize(item.hash()?);
            let key = item.get_type().hash()?;
            map.insert(key, item.unbind());
        }
        Ok(MetadataSet {
            map,
            hash: hasher.finish(),
        })
    }
}

impl Hash for MetadataSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for MetadataSet {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl MetadataSet {
    pub fn clone_ref(&self, py: Python) -> Self {
        let mut map = HashMap::new();
        for (key, value) in self.map.iter() {
            map.insert(*key, value.clone_ref(py));
        }
        Self {
            map,
            hash: self.hash,
        }
    }
}

impl ToPyObject for MetadataSet {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}

#[pymethods]
impl MetadataSet {
    pub fn get<'py>(
        slf: PyRef<'py, Self>,
        type_info: Bound<'py, PyType>,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        let key = type_info.hash()?;
        if let Some(o) = slf.map.get(&key) {
            let py = slf.py();
            Ok(Some(o.into_py(py).into_bound(py)))
        } else {
            Ok(None)
        }
    }

    pub fn __getitem__<'py>(
        slf: PyRef<'py, Self>,
        type_info: Bound<'py, PyType>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let key = type_info.hash()?;
        if let Some(o) = slf.map.get(&key) {
            let py = slf.py();
            Ok(o.into_py(py).into_bound(py))
        } else {
            Err(PyKeyError::new_err(format!(
                "Does not contain object of type {}",
                type_info.repr()?
            )))
        }
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let mut repr = String::new();
        repr.push('(');

        let mut first = true;

        for qualifier in self.map.values() {
            if !first {
                repr.push_str(", ");
            }
            let s = qualifier.bind(py).repr()?;
            repr.push_str(s.to_str()?);
            first = false;
        }

        repr.push(')');
        Ok(repr)
    }

    /// If this metadata is subset of the other metadata
    pub fn issubset(&self, metadata: &MetadataSet) -> bool {
        self.map.keys().all(|k| metadata.map.contains_key(k))
    }

    /// If this metadata is superset of the other metadata
    pub fn issuperset(&self, metadata: &MetadataSet) -> bool {
        metadata.issubset(self)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[pyclass(module = "composify.core.metadata", eq)]
#[derive(Debug, Default)]
pub struct Qualifiers {
    qualifiers: Vec<Py<PyAny>>,
    hash: u64,
}

#[pymethods]
impl Qualifiers {
    #[new]
    pub fn __new__(items: Vec<Bound<PyAny>>) -> PyResult<Self> {
        let mut hasher = DefaultHasher::default();
        let mut qualifiers = Vec::new();
        for p in items {
            hasher.write_isize(p.hash()?);
            qualifiers.push(p.unbind());
        }
        Ok(Qualifiers {
            qualifiers,
            hash: hasher.finish(),
        })
    }

    pub fn __repr__(&self, py: Python) -> PyResult<String> {
        let mut repr = String::new();
        repr.push('(');

        let mut first = true;

        for qualifier in self.qualifiers.iter() {
            if !first {
                repr.push_str(", ");
            }
            let s = qualifier.bind(py).repr()?;
            repr.push_str(s.to_str()?);
            first = false;
        }

        repr.push(')');
        Ok(repr)
    }

    pub fn __hash__(&self) -> PyResult<u64> {
        Ok(self.hash)
    }

    pub fn qualify(&self, py: Python<'_>, attrs: &MetadataSet) -> PyResult<bool> {
        let args = PyTuple::new_bound(py, [attrs.clone_ref(py).into_py(py)]);
        for q in self.qualifiers.iter() {
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

impl Qualifiers {
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Qualifiers {
            qualifiers: self.qualifiers.iter().map(|a| a.clone_ref(py)).collect(),
            hash: self.hash,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.qualifiers.is_empty()
    }
}

impl Hash for Qualifiers {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl ToPyObject for Qualifiers {
    fn to_object(&self, py: Python<'_>) -> pyo3::Py<PyAny> {
        self.clone_ref(py).into_py(py)
    }
}

impl PartialEq for Qualifiers {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Qualifiers {}

#[pyclass(module = "composify.core.metadata", frozen)]
pub struct AttributeQualifier(pub Py<MetadataSet>);

#[pymethods]
impl AttributeQualifier {
    pub fn qualify(&self, attrs: &MetadataSet) -> bool {
        self.0.get().issubset(attrs)
    }
}

impl AttributeQualifier {
    pub fn clone_ref(&self, py: Python) -> Self {
        Self(self.0.clone_ref(py))
    }
}

impl ToPyObject for AttributeQualifier {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}
