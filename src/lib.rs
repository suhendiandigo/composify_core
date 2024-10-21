use pyo3::prelude::*;

mod metadata;
mod registry;
mod resolution;
mod rules;
mod type_info;

/// The core module for composify written in rust.
#[pymodule]
fn composify(m: &Bound<'_, PyModule>) -> PyResult<()> {
    rules(m)?;
    registry(m)?;
    metadata(m)?;
    m.add_class::<type_info::TypeInfo>()?;
    Ok(())
}

fn rules(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new_bound(py, "rules")?;
    m.add_class::<rules::Dependency>()?;
    m.add_class::<rules::DependenciesIter>()?;
    m.add_class::<rules::Dependencies>()?;
    m.add_class::<rules::Rule>()?;
    py.import_bound("sys")?
        .getattr("modules")?
        .set_item("composify.rules", m)?;
    Ok(())
}

fn registry(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new_bound(py, "registry")?;
    m.add_class::<registry::RuleRegistry>()?;
    py.import_bound("sys")?
        .getattr("modules")?
        .set_item("composify.registry", m)?;
    Ok(())
}

fn metadata(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent_module.py();
    let m = PyModule::new_bound(py, "set")?;
    m.add_class::<metadata::MetadataSet>()?;
    py.import_bound("sys")?
        .getattr("modules")?
        .set_item("composify.metadata.set", m)?;
    Ok(())
}
