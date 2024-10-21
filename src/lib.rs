use pyo3::prelude::*;
use pyo3::types::{PyFunction, PyType, PyMapping, PyString};

/// The core module for composify written in rust.
#[pymodule]
mod composify_core {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use super::*;

    #[pymodule(submodule)]
    mod rules {
        use super::*;

        /// Hack: workaround for https://github.com/PyO3/pyo3/issues/759
        #[pymodule_init]
        fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
            Python::with_gil(|py| {
                py.import_bound("sys")?
                    .getattr("modules")?
                    .set_item("composify_core.rules", m)
            })
        }

        #[pyclass(get_all, frozen)]
        #[derive(Debug)]
        struct Dependency {
            name: String,
            typing: Py<PyType>,
        }

        impl ToPyObject for Dependency {
            fn to_object(&self, py: pyo3::Python<'_>) -> pyo3::Py<PyAny> {
                let d = Dependency {
                    name: self.name.clone(),
                    typing: self.typing.clone_ref(py)
                };
                d.into_py(py)
            }
        }

        impl Dependency {
            fn clone_ref(&self, py: Python<'_>) -> Dependency {
                Dependency {
                    name: self.name.clone(),
                    typing: self.typing.clone_ref(py)
                }
            }
        }

        #[pymethods]
        impl Dependency {
            #[new]
            fn new(name: Bound<'_, PyString>, typing: Bound<'_, PyType>) -> PyResult<Self> {
                Ok(Dependency{
                    name: String::from(name.to_str()?), 
                    typing: typing.unbind()
                })
            }

            fn __repr__(&self) -> String {
                format!("Dependency({}, type={})", &self.name, &self.typing)
            }

            fn __hash__(slf: PyRef<'_, Self>) -> PyResult<u64> {
                let mut hasher = DefaultHasher::new();
                slf.name.hash(&mut hasher);
                let t = slf.typing.bind(slf.py());
                hasher.write_isize(t.hash()?);
                Ok(hasher.finish())
            }
        }

        #[pyclass]
        struct DependenciesIter {
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

        #[pyclass(frozen)]
        #[derive(Debug)]
        struct Dependencies {
            dependencies: Vec<Dependency>,
        }

        #[pymethods]
        impl Dependencies {
            #[new]
            fn new(parameters: Bound<'_, PyMapping>) -> PyResult<Self> {
                let mut result = Vec::new();
                let mut iter = parameters.items()?.iter()?;
                while let Some(py_element) = iter.next() {
                    let py_tuple = py_element?;
                    let name = py_tuple.get_item(0)?.downcast_into::<PyString>()?;
                    let typing = py_tuple.get_item(1)?.downcast_into::<PyType>()?;
                    result.push(Dependency::new(name, typing)?);
                }
                result.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(Dependencies{
                    dependencies: result,
                })
            }

            fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<DependenciesIter>> {
                let py = slf.py();
                let v: Vec<Dependency> = (&slf.dependencies).into_iter().map(|d| d.clone_ref(py)).collect();
                let iter = DependenciesIter {
                    inner: v.into_iter(),
                };
                Py::new(slf.py(), iter)
            }

            fn __repr__(&self) -> String {
                format!("Dependencies({})", self.dependencies.len())
            }

            fn __hash__(slf: PyRef<'_, Self>) -> PyResult<u64> {
                let mut hasher = DefaultHasher::new();
                let py = slf.py();
                for d in &slf.dependencies {
                    d.name.hash(&mut hasher);
                    let t = d.typing.bind(py);
                    hasher.write_isize(t.hash()?);
                }
                Ok(hasher.finish())
            }
        }

        impl ToPyObject for Dependencies {
            fn to_object(&self, py: pyo3::Python<'_>) -> pyo3::Py<PyAny> { 
                let v: Vec<Dependency> = (&self.dependencies).into_iter().map(|d| d.clone_ref(py)).collect();
                let d = Dependencies {
                    dependencies: v
                };
                d.into_py(py)
            }
        }

        #[pyclass(get_all, frozen)]
        struct Rule {
            function: Py<PyFunction>,
            canonical_name: String,
            output_type: Py<PyType>,
            dependencies: Dependencies,
            priority: i32,
            is_async: bool,
            is_optional: bool,
        }

        #[pymethods]
        impl Rule {
            #[new]
            fn new(
                function: Bound<'_, PyFunction>,
                canonical_name: String,
                output_type: Bound<'_, PyType>,
                dependencies: Bound<'_, PyMapping>,
                priority: i32,
                is_async: bool,
                is_optional: bool,
            ) -> PyResult<Self> {
                Ok(Self{
                    function: function.into(),
                    canonical_name,
                    output_type: output_type.into(),
                    dependencies: Dependencies::new(dependencies)?,
                    priority,
                    is_async,
                    is_optional,
                })
            }

            fn __repr__(&self) -> String {
                format!("Rule({}, {}, out={}, priority={}, is_async={}, is_optional={})", self.canonical_name, self.function, self.output_type, self.priority, self.is_async, self.is_optional)
            }

            fn __hash__(slf: PyRef<'_, Self>) -> PyResult<u64> {
                let mut hasher = DefaultHasher::new();
                let py = slf.py();
                slf.canonical_name.hash(&mut hasher);
                slf.priority.hash(&mut hasher);
                slf.is_async.hash(&mut hasher);
                slf.is_optional.hash(&mut hasher);
                {
                    let t = slf.function.bind(py);
                    hasher.write_isize(t.hash()?);
                }
                {
                    let t = slf.output_type.bind(py);
                    hasher.write_isize(t.hash()?);
                }
                {
                    for d in &slf.dependencies.dependencies {
                        d.name.hash(&mut hasher);
                        let t = d.typing.bind(py);
                        hasher.write_isize(t.hash()?);
                    }
                }
                Ok(hasher.finish())
            }
        }
    }
}
