use std::{
    fmt::{Display, Write},
    hash::{DefaultHasher, Hash, Hasher},
};

use pyo3::{exceptions::PyIndexError, prelude::*, types::PyMapping};

use crate::{rules::Rule, type_info::TypeInfo};

#[pyclass(get_all, frozen, eq, hash, module = "composify.core.solutions")]
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
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
#[derive(Default, Debug, Clone)]
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

impl SolutionArgsCollection {
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
    #[pyo3(signature = (args=None))]
    pub fn __new__(args: Option<Bound<PyMapping>>) -> PyResult<Self> {
        match args {
            Some(args) => {
                if args.is_empty()? {
                    return Ok(Self::default());
                }
                let mut solution_args = Vec::new();
                for arg in args.items()?.iter() {
                    let name: String = arg.get_item(0)?.to_string();
                    let solution = arg.get_item(1)?.downcast::<Solution>()?.get().clone();
                    solution_args.push(SolutionArg { name, solution });
                }
                Ok(Self::new(solution_args))
            }
            None => Ok(Self::default()),
        }
    }

    pub fn __iter__(&self, py: Python) -> PyResult<Py<SolutionArgsCollectionIter>> {
        let iter = SolutionArgsCollectionIter {
            inner: self.clone().0.into_iter(),
        };
        Py::new(py, iter)
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    pub fn __getitem__(&self, i: usize) -> PyResult<SolutionArg> {
        match self.0.get(i) {
            Some(val) => Ok(val.clone()),
            None => Err(PyIndexError::new_err(format!("Index out of range: {}", i))),
        }
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
        f.write_str("SolutionArgsCollection{")?;

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
#[derive(Debug, Clone)]
pub struct Solution {
    pub rule: Rule,
    pub args: SolutionArgsCollection,
}

#[pymethods]
impl Solution {
    #[new]
    #[pyo3(signature = (rule, args=None))]
    pub fn __new__(rule: Bound<Rule>, args: Option<Bound<PyMapping>>) -> PyResult<Self> {
        Ok(Self {
            rule: rule.get().clone(),
            args: if let Some(args) = args {
                SolutionArgsCollection::__new__(Some(args))?
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
        slf.rule.output_type.clone()
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
