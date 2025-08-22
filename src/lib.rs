use pyo3::prelude::*;

mod errors;
mod metadata;
mod registry;
mod rules;
mod solutions;
mod solve_parameters;
mod solver;
mod type_info;

/// The core module for composify written in rust.
#[pymodule]
#[pyo3(name = "core")]
fn core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add_class::<type_info::TypeInfo>()?;
    m.add_class::<rules::Dependency>()?;
    m.add_class::<rules::DependenciesIter>()?;
    m.add_class::<rules::Dependencies>()?;
    m.add_class::<rules::Rule>()?;
    m.add_class::<registry::RuleRegistry>()?;
    m.add_class::<metadata::MetadataSet>()?;
    m.add_class::<solutions::SolutionArg>()?;
    m.add_class::<solutions::SolutionArgsCollection>()?;
    m.add_class::<solutions::Solution>()?;
    m.add_class::<solve_parameters::SolveCardinality>()?;
    m.add_class::<solve_parameters::SolveSpecificity>()?;
    m.add_class::<solve_parameters::SolveParameter>()?;
    m.add("SolvingError", py.get_type::<solver::SolvingError>())?;
    m.add_class::<solver::Solver>()?;

    Ok(())
}
