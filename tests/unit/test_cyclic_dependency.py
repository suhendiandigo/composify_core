from dataclasses import dataclass

from pytest import raises

from composify.errors import CyclicDependencyError, SolveFailureError
from composify.rules import collect_rules, rule
from tests.utils import create_rule_solver


@dataclass(frozen=True)
class A:
    value: int


@dataclass(frozen=True)
class B:
    value: int


@rule
def create_a(param: A) -> B:
    return B(param.value * 2)


@rule
def create_b(param: B) -> A:
    return A(param.value * 2)


rules = collect_rules()


@rule
def default_a() -> A:
    return A(5)


rules_2 = collect_rules()


def test_raises_cyclic_dependency():
    resolver = create_rule_solver(*rules)
    with raises(SolveFailureError) as exc:
        list(resolver.solve_for(B))
    assert exc.value.contains(CyclicDependencyError)


def test_cyclic_dependency_but_okay():
    resolver = create_rule_solver(*rules_2)
    plans = list(resolver.solve_for(B))
    assert len(plans) > 0
