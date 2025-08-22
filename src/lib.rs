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
fn composify(m: &Bound<'_, PyModule>) -> PyResult<()> {
    core(m)?;
    Ok(())
}

fn core(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new(py, "core")?;
    rules(&m)?;
    registry(&m)?;
    metadata(&m)?;
    solutions(&m)?;
    solver(&m)?;
    m.add_class::<type_info::TypeInfo>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("composify.core", m)?;
    Ok(())
}

fn rules(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new(py, "rules")?;
    m.add_class::<rules::Dependency>()?;
    m.add_class::<rules::DependenciesIter>()?;
    m.add_class::<rules::Dependencies>()?;
    m.add_class::<rules::Rule>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("composify.core.rules", m)?;
    Ok(())
}

fn registry(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new(py, "registry")?;
    m.add_class::<registry::RuleRegistry>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("composify.core.registry", m)?;
    Ok(())
}

fn metadata(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new(py, "set")?;
    m.add_class::<metadata::MetadataSet>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("composify.core.metadata", m)?;
    Ok(())
}

fn solutions(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new(py, "solutions")?;
    m.add_class::<solutions::SolutionArg>()?;
    m.add_class::<solutions::SolutionArgsCollection>()?;
    m.add_class::<solutions::Solution>()?;
    m.add_class::<solve_parameters::SolveCardinality>()?;
    m.add_class::<solve_parameters::SolveSpecificity>()?;
    m.add_class::<solve_parameters::SolveParameter>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("composify.core.solutions", m)?;
    Ok(())
}

fn solver(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new(py, "solver")?;
    m.add("SolvingError", py.get_type::<solver::SolvingError>())?;
    m.add_class::<solver::Solver>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("composify.core.solver", m)?;
    Ok(())
}
