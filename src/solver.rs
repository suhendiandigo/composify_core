use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyTuple};

use crate::{
    errors,
    registry::RuleRegistry,
    solutions::{Solution, SolutionArg, SolutionArgsCollection},
    solve_parameters::SolveCardinality,
    type_info::TypeInfo,
};

#[derive(Clone, PartialEq, Eq, Debug)]
struct ExecutionStep<'a> {
    name: &'a str,
    target: &'a TypeInfo,
}

type ExecutionStack<'a> = Vec<ExecutionStep<'a>>;

struct StepRaii<'a>(Rc<RefCell<ExecutionStack<'a>>>);

impl<'a> StepRaii<'a> {
    fn new(step: ExecutionStep<'a>, stack: Rc<RefCell<ExecutionStack<'a>>>) -> Self {
        stack.borrow_mut().push(step);
        Self(stack)
    }
}

impl Drop for StepRaii<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().pop();
    }
}

fn clone_stack<'a>(stack: Ref<ExecutionStack<'a>>) -> ExecutionStack<'a> {
    stack.iter().cloned().collect()
}

#[derive(Clone, Default)]
pub struct SolutionsMemo(Arc<RwLock<HashMap<TypeInfo, Vec<Solution>>>>);

impl SolutionsMemo {
    pub fn read_memo(&self, py: Python, t: &TypeInfo) -> Option<Vec<Solution>> {
        match self.0.read() {
            Ok(map) => map
                .get(t)
                .map(|memo| memo.iter().map(|s| s.clone_ref(py)).collect()),
            Err(_) => None,
        }
    }

    pub fn save_memo(&self, py: Python, t: &TypeInfo, solutions: Vec<Solution>) {
        if let Ok(mut map) = self.0.write() {
            map.insert(t.clone_ref(py), solutions);
        }
    }
}

#[derive(Debug)]
pub enum SolvingErrorReason {
    CyclicDependency,
    NoSolution,
    NotExclusive(Vec<Solution>),
}

create_exception!(composify.core.solver, SolvingError, PyException);

fn permutate_candidates(
    py: Python,
    candidates: Vec<SolutionArgCandidate>,
) -> Result<Vec<SolutionArgsCollection>, SolvingErrorReason> {
    let mut curr_iteration: Vec<SolutionArgsCollection>;
    let mut next_iteration: Vec<SolutionArgsCollection> = Vec::new();
    let mut iter = candidates.into_iter();
    if let Some(c) = iter.next() {
        for s in c.solutions {
            next_iteration.push(SolutionArgsCollection::new(vec![SolutionArg {
                name: c.name.clone(),
                solution: s,
            }]));
        }
    } else {
        return Err(SolvingErrorReason::NoSolution);
    }

    for c in iter {
        curr_iteration = Vec::new();
        curr_iteration.extend(next_iteration);
        next_iteration = Vec::new();
        for args in curr_iteration.into_iter() {
            for solution in c.solutions.iter() {
                let mut args = args.clone_ref(py);
                args.0.push(SolutionArg {
                    name: c.name.to_string(),
                    solution: solution.clone_ref(py),
                });
                next_iteration.push(args)
            }
        }
    }

    Ok(next_iteration)
}

pub struct _Solver<'a> {
    solver: &'a Solver,
    py: Python<'a>,
    execution_stack: Rc<RefCell<ExecutionStack<'a>>>,
    errors: RefCell<Vec<(ExecutionStack<'a>, SolvingErrorReason)>>,
}

pub struct SolutionArgCandidate {
    pub name: String,
    pub solutions: Vec<Solution>,
}

impl<'a> _Solver<'a> {
    fn new(solver: &'a Solver, py: Python<'a>) -> Self {
        Self {
            solver,
            py,
            execution_stack: Rc::new(RefCell::new(Vec::new())),
            errors: RefCell::new(Vec::new()),
        }
    }

    fn push_error(&self, error: SolvingErrorReason) {
        self.errors
            .borrow_mut()
            .push((clone_stack(self.execution_stack.borrow()), error));
    }

    fn push_stack(&'a self, name: &'a str, target: &'a TypeInfo) -> Option<StepRaii> {
        let step = ExecutionStep { name, target };
        if self.execution_stack.borrow().len() > 5 {
            println!("{}", step.target.type_hash);
            self.execution_stack
                .borrow()
                .iter()
                .map(|f| println!("{}", f.target == step.target))
                .for_each(drop);
            return None;
        }
        if self
            .execution_stack
            .borrow()
            .iter()
            .any(|f| f.target == step.target)
        {
            let _raii = StepRaii::new(step, self.execution_stack.clone());
            self.push_error(SolvingErrorReason::CyclicDependency);
            None
        } else {
            Some(StepRaii::new(step, self.execution_stack.clone()))
        }
    }

    fn solve_for<'b: 'a>(
        &'b self,
        name: &'b str,
        target: &'b TypeInfo,
    ) -> PyResult<Option<Vec<Solution>>> {
        if let Some(solutions) = self.solver.memo.read_memo(self.py, target) {
            return Ok(Some(solutions));
        }
        // If unnamed (_), value is immediately dropped.
        let _pop_on_drop = self.push_stack(name, target);
        if _pop_on_drop.is_none() {
            return Ok(None);
        }
        let rules = if let Some(rules) = self.solver.rules.get(self.py, target)? {
            rules
        } else {
            self.push_error(SolvingErrorReason::NoSolution);
            return Ok(None);
        };
        let mut solutions = Vec::new();
        'rule: for rule in rules {
            if rule.dependencies.is_empty() {
                solutions.push(Solution {
                    rule: rule.clone_ref(self.py),
                    args: SolutionArgsCollection::default(),
                });
            } else {
                let mut args: Vec<SolutionArgCandidate> = Vec::new();
                for dependency in rule.dependencies.iter() {
                    match self.solve_for(dependency.name.as_str(), &dependency.typing)? {
                        Some(solutions) => {
                            args.push(SolutionArgCandidate {
                                name: dependency.name.to_string(),
                                solutions,
                            });
                        }
                        None => continue 'rule,
                    }
                }
                match permutate_candidates(self.py, args) {
                    Ok(args) => {
                        for args in args {
                            solutions.push(Solution {
                                rule: rule.clone_ref(self.py),
                                args,
                            });
                        }
                    }
                    Err(e) => self.push_error(e),
                }
            }
        }
        if solutions.is_empty() {
            self.push_error(SolvingErrorReason::NoSolution);
            Ok(None)
        } else {
            let solutions = match target.solve_parameter.cardinality {
                SolveCardinality::Exhaustive => solutions,
                SolveCardinality::Single => match solutions.into_iter().next() {
                    Some(r) => vec![r],
                    None => Vec::new(),
                },
                SolveCardinality::Exclusive => {
                    if solutions.len() > 1 {
                        self.push_error(SolvingErrorReason::NotExclusive(solutions));
                        return Ok(None);
                    }
                    solutions
                }
            };
            self.solver.memo.save_memo(
                self.py,
                target,
                solutions.iter().map(|s| s.clone_ref(self.py)).collect(),
            );
            Ok(Some(solutions))
        }
    }
}

#[pyclass(module = "composify.core.solver")]
#[derive(Clone)]
pub struct Solver {
    pub rules: Arc<RuleRegistry>,
    pub memo: SolutionsMemo,
}

fn make_trace_tuple<'a>(py: Python<'a>, stack: &ExecutionStack) -> Bound<'a, PyTuple> {
    let mut steps: Vec<Bound<PyTuple>> = Vec::new();
    for step in stack {
        steps.push(PyTuple::new_bound(
            py,
            [step.name.to_object(py), step.target.to_object(py)],
        ));
    }
    PyTuple::new_bound(py, steps)
}

fn make_py_error(py: Python, stack: &ExecutionStack, reason: &SolvingErrorReason) -> PyErr {
    let traces = make_trace_tuple(py, stack);
    match reason {
        SolvingErrorReason::NoSolution => {
            errors::NoSolutionError::new_err(PyTuple::new_bound(py, [traces]).unbind())
        }
        SolvingErrorReason::CyclicDependency => {
            errors::CyclicDependencyError::new_err(PyTuple::new_bound(py, [traces]).unbind())
        }
        SolvingErrorReason::NotExclusive(solutions) => errors::NotExclusiveError::new_err(
            PyTuple::new_bound(py, [PyTuple::new_bound(py, solutions), traces]).unbind(),
        ),
    }
}

#[pymethods]
impl Solver {
    #[new]
    pub fn __new__(py: Python, registry: &RuleRegistry) -> PyResult<Self> {
        Ok(Self {
            rules: Arc::new(registry.clone_ref(py)),
            memo: SolutionsMemo::default(),
        })
    }

    pub fn solve_for(&self, target: Bound<PyAny>) -> PyResult<Vec<Solution>> {
        let py = target.py();
        let t = TypeInfo::parse(target)?;
        let solver = _Solver::new(self, py);
        if let Some(solutions) = solver.solve_for("__root__", &t)? {
            Ok(solutions)
        } else {
            let errors: Vec<PyErr> = solver
                .errors
                .borrow()
                .iter()
                .map(|(s, r)| make_py_error(py, s, r))
                .collect();
            Err(errors::SolveFailureError::new_err(errors))
        }
    }
}

impl ToPyObject for Solver {
    fn to_object(&self, py: Python) -> PyObject {
        self.clone().into_py(py)
    }
}
