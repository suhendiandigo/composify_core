use std::fmt::{Display, Write};

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

#[pymethods]
impl SolutionArg {
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

impl Display for SolutionArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SolutionArg({}: {})", self.name, self.solution)
    }
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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[pymethods]
impl SolutionArgsCollection {
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

impl Display for SolutionArgsCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;

        let mut first = true;

        for solution_arg in self.0.iter() {
            if !first {
                f.write_str(", ")?;
            }
            f.write_str(&solution_arg.name)?;
            f.write_str(": ")?;
            solution_arg.solution.fmt(f)?;
            first = false;
        }

        f.write_char('}')?;
        Ok(())
    }
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

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Solution(rule={}, arguments={})",
            self.rule, self.args
        ))
    }

    pub fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            write!(
                f,
                "Solution({}, rule={})",
                self.rule.output_type.to_type_string(),
                self.rule.canonical_name
            )
        } else {
            write!(
                f,
                "Solution({}, rule={}, arguments={})",
                self.rule.output_type.to_type_string(),
                self.rule.canonical_name,
                self.args
            )
        }
    }
}

impl ToPyObject for Solution {
    fn to_object(&self, py: Python) -> PyObject {
        self.clone_ref(py).into_py(py)
    }
}
