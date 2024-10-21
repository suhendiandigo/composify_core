use pyo3::exceptions::PyKeyError;
use pyo3::types::PyType;
use pyo3::PyObject;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher, Hash};


#[pyclass(get_all, frozen, module="composify.metadata.set", eq, hash, subclass)]
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
    pub fn clone_ref(&self, py: Python<'_>) -> MetadataSet {
        let mut map = HashMap::new();
        for (key, value) in self.map.iter() {
            map.insert(*key, value.clone_ref(py));
        }
        MetadataSet {
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
    pub fn get<'py>(slf: PyRef<'py, Self>, type_info: Bound<'py, PyType>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let key = type_info.hash()?;
        if let Some(o) = slf.map.get(&key) {
            let py = slf.py();
            Ok(Some(o.into_py(py).into_bound(py)))
        } else {
            Ok(None)
        }
    }

    pub fn __getitem__<'py>(slf: PyRef<'py, Self>, type_info: Bound<'py, PyType>) -> PyResult<Bound<'py, PyAny>> {
        let key = type_info.hash()?;
        if let Some(o) = slf.map.get(&key) {
            let py = slf.py();
            Ok(o.into_py(py).into_bound(py))
        } else {
            Err(PyKeyError::new_err(format!("Does not contain object of type {}", type_info.repr()?)))
        }
    }
}
