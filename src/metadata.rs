use pyo3::exceptions::PyKeyError;
use pyo3::types::{PyBool, PyTuple, PyType};
use pyo3::PyObject;
use pyo3::{intern, prelude::*};
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::slice::Iter;

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

    pub fn iter(&self) -> Values<isize, PyObject> {
        self.map.values()
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

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
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

impl Display for MetadataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;

        let mut first = true;

        for item in self.map.values() {
            if !first {
                f.write_str(", ")?;
            }
            item.fmt(f)?;
            first = false;
        }

        f.write_char(')')?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Qualifier {
    inner: PyObject,
    inner_self: Option<PyObject>,
}

pub const QUALIFY_METHOD_NAME: &str = "qualify";

impl Qualifier {
    pub fn new(qualifier: Bound<PyAny>) -> Self {
        if let Ok(func) = qualifier.getattr(intern!(qualifier.py(), QUALIFY_METHOD_NAME)) {
            Self {
                inner: func.unbind(),
                inner_self: Some(qualifier.unbind()),
            }
        } else {
            Self {
                inner: qualifier.unbind(),
                inner_self: None,
            }
        }
    }

    /// Invoke the inner python qualifier object.
    /// Takes a reference to a bound python tuple as args.
    pub fn call(&self, args: &Bound<PyTuple>) -> PyResult<bool> {
        let q = self.inner.bind(args.py()).call1(args)?;
        let q = q.downcast_into::<PyBool>()?;
        Ok(q.is_true())
    }

    /// Invoke the inner python qualifier for an attribute set.
    pub fn qualify(&self, py: Python<'_>, attrs: &MetadataSet) -> PyResult<bool> {
        let args = PyTuple::new_bound(py, [attrs.clone_ref(py).into_py(py)]);
        self.call(&args)
    }

    pub fn clone_ref(&self, py: Python) -> Self {
        Self {
            inner: self.inner.clone_ref(py),
            inner_self: self.inner_self.as_ref().map(|s| s.clone_ref(py)),
        }
    }
}

impl Display for Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(inner_self) = &self.inner_self {
            inner_self.fmt(f)
        } else {
            self.inner.fmt(f)
        }
    }
}

#[pyclass(frozen, eq, module = "composify.core.metadata")]
#[derive(Debug, Default)]
pub struct Qualifiers {
    qualifiers: Vec<Qualifier>,
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
            qualifiers.push(Qualifier::new(p));
        }
        Ok(Self {
            qualifiers,
            hash: hasher.finish(),
        })
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    pub fn __hash__(&self) -> PyResult<u64> {
        Ok(self.hash)
    }

    pub fn qualify(&self, py: Python, attrs: &MetadataSet) -> PyResult<bool> {
        let args = PyTuple::new_bound(py, [attrs.clone_ref(py).into_py(py)]);
        for q in self.qualifiers.iter() {
            if !q.call(&args)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl Qualifiers {
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            qualifiers: self.qualifiers.iter().map(|a| a.clone_ref(py)).collect(),
            hash: self.hash,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.qualifiers.is_empty()
    }

    pub fn iter(&self) -> Iter<Qualifier> {
        self.qualifiers.iter()
    }
}

impl Display for Qualifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;

        let mut first = true;

        for qualifier in self.qualifiers.iter() {
            if !first {
                f.write_str(", ")?;
            }
            qualifier.fmt(f)?;
            first = false;
        }

        f.write_char(')')?;
        Ok(())
    }
}

impl Hash for Qualifiers {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for Qualifiers {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Qualifiers {}

impl ToPyObject for Qualifiers {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}
