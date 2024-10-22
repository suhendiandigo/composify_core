use std::{
    cell::{Ref, RefCell},
    collections::{BinaryHeap, HashMap},
    rc::Rc,
    sync::{Arc, RwLock},
};

use pyo3::{create_exception, exceptions::PyException, prelude::*};

use crate::{
    registry::RuleRegistry,
    rules::Rule,
    solutions::{Solution, SolutionArg, SolutionArgsCollection},
    solve_parameters::SolveCardinality,
    type_info::TypeInfo,
};

#[derive(Clone, PartialEq, Eq)]
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
            Ok(map) => {
                map.get(t).map(|memo| memo.iter().map(|s| s.clone_ref(py)).collect())
            }
            Err(_) => None,
        }
    }

    pub fn save_memo(&self, py: Python, t: &TypeInfo, solutions: Vec<Solution>) {
        if let Ok(mut map) = self.0.write() {
            map.insert(t.clone_ref(py), solutions);
        }
    }
}

pub enum _SolvingError {
    CyclicDependency,
    NoSolution,
    NotExclusive,
}

create_exception!(composify.core.solver, SolvingError, PyException);

fn permutate_candidates(
    py: Python,
    candidates: Vec<SolutionArgCandidate>,
) -> Result<Vec<SolutionArgsCollection>, _SolvingError> {
    let mut curr_iteration: Vec<SolutionArgsCollection>;
    let mut next_iteration: Vec<SolutionArgsCollection> = Vec::new();
    let mut iter = candidates.into_iter();
    if let Some(c) = iter.next() {
        for s in c.solutions {
            next_iteration.push(SolutionArgsCollection(vec![SolutionArg {
                name: c.name.clone(),
                solution: s,
            }]));
        }
    } else {
        return Err(_SolvingError::NoSolution);
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
    errors: RefCell<Vec<(ExecutionStack<'a>, _SolvingError)>>,
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

    fn push_error(&self, error: _SolvingError) {
        self.errors
            .borrow_mut()
            .push((clone_stack(self.execution_stack.borrow()), error));
    }

    fn iterate_rule(
        &'a self,
        rules: &'a BinaryHeap<Rule>,
        cardinality: &SolveCardinality,
    ) -> Option<Vec<&'a Rule>> {
        Some(match cardinality {
            SolveCardinality::Exhaustive => rules.iter().collect(),
            SolveCardinality::Single => match rules.iter().next() {
                Some(r) => vec![r],
                None => Vec::new(),
            },
            SolveCardinality::Exclusive => {
                if rules.len() > 1 {
                    self.push_error(_SolvingError::NotExclusive);
                    return None;
                }
                rules.iter().collect()
            }
        })
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
            self.push_error(_SolvingError::CyclicDependency);
            None
        } else {
            Some(StepRaii::new(step, self.execution_stack.clone()))
        }
    }

    fn solve_for<'b: 'a>(&'b self, name: &'b str, target: &'b TypeInfo) -> Option<Vec<Solution>> {
        if let Some(solutions) = self.solver.memo.read_memo(self.py, target) {
            return Some(solutions);
        }
        // If unnamed (_), value is immediately dropped.
        let _pop_on_drop = self.push_stack(name, target)?;
        if let Some(rules) = self.solver.rules.get(target) {
            let mut solutions = Vec::new();
            'rule: for rule in self.iterate_rule(rules, &target.solve_parameter.cardinality)? {
                if rule.dependencies.is_empty() {
                    solutions.push(Solution {
                        rule: rule.clone_ref(self.py),
                        args: SolutionArgsCollection::default(),
                    });
                } else {
                    let mut args: Vec<SolutionArgCandidate> = Vec::new();
                    for dependency in rule.dependencies.iter() {
                        match self.solve_for(dependency.name.as_str(), &dependency.typing) {
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
                self.push_error(_SolvingError::NoSolution);
                None
            } else {
                self.solver.memo.save_memo(
                    self.py,
                    target,
                    solutions.iter().map(|s| s.clone_ref(self.py)).collect(),
                );
                Some(solutions)
            }
        } else {
            None
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Solver {
    pub rules: Arc<RuleRegistry>,
    pub memo: SolutionsMemo,
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
        if let Some(solutions) = solver.solve_for("__root__", &t) {
            Ok(solutions)
        } else {
            Err(SolvingError::new_err("Failed to solve"))
        }
    }
}

impl ToPyObject for Solver {
    fn to_object(&self, py: Python) -> PyObject {
        self.clone().into_py(py)
    }
}
