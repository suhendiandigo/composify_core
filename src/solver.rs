use std::{collections::HashMap, sync::{Arc, RwLock}};

use pyo3::{exceptions::PyRuntimeError, prelude::*};

use crate::{registry::RuleRegistry, solutions::Solution, type_info::TypeInfo};

struct ExecutionStep<'a> {
    name: &'a str,
    target: &'a TypeInfo,
}


pub struct _Solver<'a> {
    solver: &'a Solver,
    py: Python<'a>,
    execution_stack: Vec<ExecutionStep<'a>>,
}

impl<'a> _Solver<'a> {
    fn new(solver: &'a Solver, py: Python<'a>) -> Self {
        Self {
            solver,
            py,
            execution_stack: Vec::new(),
        }
    }

    fn solve_for(&mut self, name: &'a str, target: &'a TypeInfo) -> Option<Vec<Solution>> {
        self.execution_stack.push(ExecutionStep{
            name,
            target
        });

        if let Some(rules) = self.solver.rules.get(target) {
            let mut solutions = Vec::new();
            for rule in rules {
                todo!("Implement solving dependencies")
            }
            Some(solutions)
        } else {
            None
        }
    }
}


#[pyclass]
#[derive(Clone)]
pub struct Solver {
    pub rules: Arc<RuleRegistry>,
    pub memo: Arc<RwLock<HashMap<TypeInfo, Vec<Solution>>>>,
}

#[pymethods]
impl Solver {
    pub fn solve_for(&self, target: Bound<PyAny>) -> PyResult<Option<Vec<Solution>>> {
        let py = target.py();
        let t = TypeInfo::parse(target)?;
        if let Some(s) = self.read_memo(py, &t)? {
            return Ok(Some(s));
        }
        let mut s: _Solver<'_> = _Solver::new(&self, py);
        s.solve_for("__root__", &t);
        Ok(None)
    }
}

impl Solver {

    pub fn read_memo(&self, py: Python, t: &TypeInfo) -> PyResult<Option<Vec<Solution>>> {
        match self.memo.read() {
            Ok(o) => if let Some(memo) = o.get(&t) {
                return Ok(Some(memo.iter().map(|s| s.clone_ref(py)).collect()))
            } else {
                return Ok(None)
            },
            Err(e) => Err(PyRuntimeError::new_err(format!("{}", e))),
        }
    }
}

impl ToPyObject for Solver {
    fn to_object(&self, py: Python) -> PyObject {
        self.clone().into_py(py)
    }
}
