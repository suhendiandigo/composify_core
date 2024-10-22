use pyo3::prelude::*;
use pyo3::types::{PyMapping, PyString};

use std::fmt::{Display, Write};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::slice::Iter;

use crate::type_info::TypeInfo;

#[pyclass(get_all, frozen, eq, module = "composify.core.rules")]
#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub typing: TypeInfo,
}

impl ToPyObject for Dependency {
    fn to_object(&self, py: Python<'_>) -> pyo3::Py<PyAny> {
        let d = Dependency {
            name: self.name.clone(),
            typing: self.typing.clone_ref(py),
        };
        d.into_py(py)
    }
}

impl Dependency {
    fn clone_ref(&self, py: Python<'_>) -> Dependency {
        Dependency {
            name: self.name.clone(),
            typing: self.typing.clone_ref(py),
        }
    }
}

#[pymethods]
impl Dependency {
    #[new]
    fn new(name: Bound<'_, PyString>, typing: Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Dependency {
            name: String::from(name.to_str()?),
            typing: TypeInfo::parse(typing)?,
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __hash__(slf: PyRef<'_, Self>) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        slf.name.hash(&mut hasher);
        slf.typing.hash(&mut hasher);
        Ok(hasher.finish())
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dependency({}, type={})", &self.name, &self.typing,)
    }
}

impl Hash for Dependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.typing.hash(state);
    }
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.typing == other.typing
    }
}

#[pyclass(module = "composify.core.rules")]
pub struct DependenciesIter {
    inner: std::vec::IntoIter<Dependency>,
}

#[pymethods]
impl DependenciesIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Dependency> {
        slf.inner.next()
    }
}

#[pyclass(frozen, eq, module = "composify.core.rules")]
#[derive(Debug)]
pub struct Dependencies {
    pub dependencies: Vec<Dependency>,
}

#[pymethods]
impl Dependencies {
    #[new]
    fn new(parameters: Bound<'_, PyMapping>) -> PyResult<Self> {
        let mut result = Vec::new();
        for py_element in parameters.items()?.iter()?.flatten() {
            let name = py_element.get_item(0)?.downcast_into::<PyString>()?;
            let typing = py_element.get_item(1)?;
            result.push(Dependency::new(name, typing)?);
        }
        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Dependencies {
            dependencies: result,
        })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<DependenciesIter>> {
        let py = slf.py();
        let d = slf.clone_ref(py);
        let iter = DependenciesIter {
            inner: d.dependencies.into_iter(),
        };
        Py::new(slf.py(), iter)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __hash__(slf: PyRef<'_, Self>) -> PyResult<u64> {
        let mut hasher = DefaultHasher::new();
        slf.hash(&mut hasher);
        Ok(hasher.finish())
    }
}

impl Dependencies {
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Dependencies {
            dependencies: self.dependencies.iter().map(|d| d.clone_ref(py)).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }

    pub fn iter(&self) -> Iter<Dependency> {
        self.dependencies.iter()
    }
}

impl Display for Dependencies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;

        let mut first = true;

        for dependency in self.dependencies.iter() {
            if !first {
                f.write_str(", ")?;
            }
            f.write_str(&dependency.name)?;
            f.write_char('=')?;
            f.write_str(&dependency.typing.to_string())?;
            first = false;
        }

        f.write_char('}')?;
        Ok(())
    }
}

impl PartialEq for Dependencies {
    fn eq(&self, other: &Self) -> bool {
        self.dependencies == other.dependencies
    }
}

impl Hash for Dependencies {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for dependency in &self.dependencies {
            dependency.hash(state);
        }
    }
}

impl ToPyObject for Dependencies {
    fn to_object(&self, py: Python<'_>) -> pyo3::Py<PyAny> {
        let v: Vec<Dependency> = self.dependencies.iter().map(|d| d.clone_ref(py)).collect();
        let d = Dependencies { dependencies: v };
        d.into_py(py)
    }
}

#[pyclass(get_all, frozen, eq, hash, module = "composify.core.rules")]
pub struct Rule {
    pub function: Py<PyAny>,
    pub canonical_name: String,
    pub output_type: TypeInfo,
    pub dependencies: Dependencies,
    pub priority: i32,
    pub is_async: bool,
}

#[pymethods]
impl Rule {
    #[new]
    pub fn new(
        py: Python,
        function: Bound<'_, PyAny>,
        canonical_name: String,
        output_type: Bound<'_, PyAny>,
        dependencies: Bound<'_, PyAny>,
        priority: i32,
        is_async: bool,
    ) -> PyResult<Self> {
        Ok(Self {
            function: function.into(),
            canonical_name,
            output_type: TypeInfo::parse(output_type)?,
            dependencies: if let Ok(dependencies) = dependencies.downcast::<PyMapping>() {
                Dependencies::new(dependencies.clone())?
            } else {
                dependencies
                    .downcast_into::<Dependencies>()?
                    .get()
                    .clone_ref(py)
            },
            priority,
            is_async,
        })
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

impl Rule {
    pub fn clone_ref(&self, py: Python<'_>) -> Self {
        Self {
            function: self.function.clone_ref(py),
            canonical_name: self.canonical_name.clone(),
            output_type: self.output_type.clone_ref(py),
            dependencies: self.dependencies.clone_ref(py),
            priority: self.priority,
            is_async: self.is_async,
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rule({}, {}, out={}, priority={}, is_async={}, dependencies={})",
            self.canonical_name,
            self.function,
            self.output_type,
            self.priority,
            self.is_async,
            self.dependencies,
        )
    }
}

impl ToPyObject for Rule {
    fn to_object(&self, py: Python<'_>) -> pyo3::Py<PyAny> {
        let r = self.clone_ref(py);
        r.into_py(py)
    }
}

impl PartialEq for Rule {
    fn eq(&self, other: &Self) -> bool {
        self.canonical_name == other.canonical_name
            && self.output_type == other.output_type
            && self.dependencies == other.dependencies
            && self.priority == other.priority
            && self.is_async == other.is_async
    }
}

impl Eq for Rule {}

impl PartialOrd for Rule {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rule {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl Hash for Rule {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.output_type.hash(state);
        self.dependencies.hash(state);
        self.is_async.hash(state);
    }
}
