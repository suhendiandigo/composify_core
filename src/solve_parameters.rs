use std::fmt::Display;

use pyo3::prelude::*;

#[pyclass(eq, eq_int, frozen, module = "composify.core.solutions")]
#[derive(PartialEq, Clone, Debug)]
pub enum SolveCardinality {
    /// Solve for all possible solutions.
    Exhaustive,
    /// Solve for a the first available solution.
    Single,
    /// Solve for an exclusive solution, raise error if multiple solutions are found.
    Exclusive,
}

impl Default for SolveCardinality {
    fn default() -> Self {
        Self::Exclusive
    }
}

impl Display for SolveCardinality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exhaustive => write!(f, "Exhaustive"),
            Self::Single => write!(f, "Single"),
            Self::Exclusive => write!(f, "Exclusive"),
        }
    }
}

#[pyclass(eq, eq_int, frozen, module = "composify.core.solutions")]
#[derive(PartialEq, Clone, Debug)]
pub enum SolveSpecificity {
    /// Solve for exact type.
    Exact,
    /// Solve allowing subclass.
    AllowSubclass,
    /// Solve allowing superclass.
    AllowSuperclass,
}

impl Default for SolveSpecificity {
    fn default() -> Self {
        Self::AllowSubclass
    }
}

impl Display for SolveSpecificity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact => write!(f, "Exact"),
            Self::AllowSubclass => write!(f, "AllowSubclass"),
            Self::AllowSuperclass => write!(f, "AllowSuperclass"),
        }
    }
}

#[pyclass(get_all, frozen, eq, module = "composify.core.solutions")]
#[derive(PartialEq, Default, Clone, Debug)]
pub struct SolveParameter {
    pub specificity: SolveSpecificity,
    pub cardinality: SolveCardinality,
}

#[pymethods]
impl SolveParameter {
    #[new]
    pub fn __new__(specificity: &SolveSpecificity, cardinality: &SolveCardinality) -> Self {
        Self {
            specificity: specificity.clone(),
            cardinality: cardinality.clone(),
        }
    }
}

impl Display for SolveParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Solve(specificity={}, cardinality={})",
            self.specificity, self.cardinality
        )
    }
}
