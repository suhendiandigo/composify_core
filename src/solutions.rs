use std::{
    fmt::{Display, Write},
    hash::{DefaultHasher, Hash, Hasher},
};

use pyo3::{
    prelude::*,
    types::{PyMapping, PyString, PyTuple},
};

use crate::{rules::Rule, type_info::TypeInfo};

#[pyclass(get_all, frozen, eq, hash, module = "composify.core.solutions")]
#[derive(Hash, PartialEq, Eq, Debug)]
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

impl PartialOrd for SolutionArg {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SolutionArg {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[pyclass(module = "composify.core.solutions")]
pub struct SolutionArgsCollectionIter {
    inner: std::vec::IntoIter<SolutionArg>,
}

#[pymethods]
impl SolutionArgsCollectionIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<SolutionArg> {
        slf.inner.next()
    }
}

#[pyclass(frozen, sequence, eq, hash, module = "composify.core.solutions")]
#[derive(Default, Debug)]
pub struct SolutionArgsCollection(pub Vec<SolutionArg>, pub u64);

impl SolutionArgsCollection {
    pub fn new(mut args: Vec<SolutionArg>) -> Self {
        args.sort();
        let mut h = 0;
        if !args.is_empty() {
            let mut hasher = DefaultHasher::default();
            args.hash(&mut hasher);
            h = hasher.finish();
        }
        Self(args, h)
    }
}

impl ToPyObject for SolutionArgsCollection {
    fn to_object(&self, py: Python) -> PyObject {
        let l: Vec<SolutionArg> = self.0.iter().map(|s| s.clone_ref(py)).collect();
        PyTuple::new_bound(py, l).unbind().into_any()
    }
}

impl SolutionArgsCollection {
    pub fn clone_ref(&self, py: Python) -> Self {
        SolutionArgsCollection(self.0.iter().map(|s| s.clone_ref(py)).collect(), self.1)
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
    #[new]
    pub fn __new__(py: Python, args: Bound<PyMapping>) -> PyResult<Self> {
        let mut solution_args = Vec::new();
        for arg in args.items()?.iter()? {
            let arg = arg?;
            let name = arg.get_item(0)?.downcast_into::<PyString>()?.to_string();
            let solution = arg.get_item(1)?.downcast::<Solution>()?.get().clone_ref(py);
            solution_args.push(SolutionArg { name, solution });
        }
        Ok(Self::new(solution_args))
    }

    fn __iter__(&self, py: Python) -> PyResult<Py<SolutionArgsCollectionIter>> {
        let iter = SolutionArgsCollectionIter {
            inner: self.clone_ref(py).0.into_iter(),
        };
        Py::new(py, iter)
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

impl PartialEq for SolutionArgsCollection {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for SolutionArgsCollection {}

impl Hash for SolutionArgsCollection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.1);
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

#[pyclass(get_all, frozen, eq, hash, module = "composify.core.solutions")]
#[derive(Debug)]
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
    #[new]
    #[pyo3(signature = (rule, args=None))]
    pub fn __new__(
        py: Python,
        rule: Bound<Rule>,
        args: Option<Bound<PyMapping>>,
    ) -> PyResult<Self> {
        Ok(Self {
            rule: rule.get().clone_ref(py),
            args: if let Some(args) = args {
                SolutionArgsCollection::__new__(py, args)?
            } else {
                SolutionArgsCollection::default()
            },
        })
    }

    #[getter]
    pub fn function(slf: PyRef<Self>) -> Bound<PyAny> {
        slf.rule.function.clone_ref(slf.py()).into_bound(slf.py())
    }

    #[getter]
    pub fn output_type(slf: PyRef<Self>) -> TypeInfo {
        slf.rule.output_type.clone_ref(slf.py())
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

impl Hash for Solution {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.args.hash(state);
    }
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        self.rule == other.rule && self.args == other.args
    }
}

impl Eq for Solution {}
