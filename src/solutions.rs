use pyo3::{
    prelude::*,
    types::{PyFunction, PyTuple},
};

use crate::rules::Rule;

#[pyclass(get_all, frozen, module = "composify.core.solutions")]
pub struct SolutionArg {
    pub name: String,
    pub solution: Solution,
}

impl SolutionArg {
    pub fn clone_ref(&self, py: Python) -> Self {
        SolutionArg {
            name: self.name.clone(),
            solution: self.solution.clone_ref(py),
        }
    }
}

impl ToPyObject for SolutionArg {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}

#[pyclass(frozen, sequence, module = "composify.core.solutions")]
#[derive(Default)]
pub struct SolutionArgsCollection(pub Vec<SolutionArg>);

impl ToPyObject for SolutionArgsCollection {
    fn to_object(&self, py: Python) -> PyObject {
        let l: Vec<SolutionArg> = self.0.iter().map(|s| s.clone_ref(py)).collect();
        PyTuple::new_bound(py, l).unbind().into_any()
    }
}

impl SolutionArgsCollection {
    pub fn clone_ref(&self, py: Python) -> Self {
        SolutionArgsCollection(self.0.iter().map(|s| s.clone_ref(py)).collect())
    }

    pub fn add(&mut self, arg: SolutionArg) {
        self.0.push(arg);
    }
}

#[pymethods]
impl SolutionArgsCollection {
    // pub fn __iter__(&self, py: Python) -> PyResult<Py<PyIterator>> {
    //     Ok(PyTuple::new_bound(py, self.0))
    // }
}

#[pyclass(get_all, frozen, module = "composify.core.solutions")]
pub struct Solution {
    pub rule: Rule,
    pub args: SolutionArgsCollection,
}

impl Solution {
    pub fn clone_ref(&self, py: Python) -> Self {
        Solution {
            rule: self.rule.clone_ref(py),
            args: self.args.clone_ref(py),
        }
    }
}

#[pymethods]
impl Solution {
    #[getter]
    pub fn function(slf: PyRef<Self>) -> Bound<PyFunction> {
        slf.rule.function.clone_ref(slf.py()).into_bound(slf.py())
    }

    #[getter]
    pub fn is_async(&self) -> bool {
        self.rule.is_async
    }

    #[getter]
    pub fn is_optional(&self) -> bool {
        self.rule.is_optional
    }
}

impl ToPyObject for Solution {
    fn to_object(&self, py: Python) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}
