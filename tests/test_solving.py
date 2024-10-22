
from dataclasses import dataclass

import pytest

from composify.core.registry import RuleRegistry
from composify.core.solver import Solver, SolvingError
from composify.rules import collect_rules, rule, as_rule


@dataclass
class A:
    value: int


@dataclass
class B:
    value: int


@rule
def example_a() -> A:
    return A(5)


@rule
def example_b(a: A) -> B:
    return B(a.value)


rules = collect_rules()

def test_solving():
    registry = RuleRegistry()
    registry.add_rules(rules)
    solver = Solver(registry)

    solutions = solver.solve_for(B)
    
    assert len(solutions) == 1
    assert solutions[0].rule == as_rule(example_b)
    assert solutions[0].args[0].name == "a"
    assert solutions[0].args[0].solution.rule == as_rule(example_a)
    assert solutions[0].args[0].solution.args == ()


@rule
def example_cyclic(b: B) -> A:
    return A(b.value)


def test_cyclic_solution():
    registry = RuleRegistry()
    registry.add_rule(as_rule(example_b))
    registry.add_rule(as_rule(example_cyclic))
    solver = Solver(registry)

    with pytest.raises(SolvingError):
        solver.solve_for(B)
