from typing import Any

from composify.core import RuleRegistry, Solution, Solver
from composify.rules import as_rule, static_rule, wraps_rule


def create_rule_solver(*rules) -> Solver:
    reg = RuleRegistry()
    reg.add_rules(as_rule(rule) for rule in rules)
    return Solver(reg)


def solution(rule: Any, **kwargs: Solution) -> Solution:
    return Solution(as_rule(rule), kwargs)


def static(value: Any, **kwargs) -> Solution:
    return Solution(static_rule("__test_static__", value, **kwargs))


class ExecutionCounter:
    def __init__(self) -> None:
        self.execution = 0

    def __call__(self, f):
        @wraps_rule(f)
        def wrapper(*args, **kwargs):
            self.execution += 1
            return f(*args, **kwargs)

        return wrapper


def _find_difference(
    result: Solution, expected: Solution, path: tuple
) -> tuple | None:
    if result.rule != expected.rule:
        return path
    r_depends = tuple(result.args)
    e_depends = tuple(expected.args)
    if len(r_depends) != len(e_depends):
        return path
    for (r_name, r_depend), (e_name, e_depend) in zip(r_depends, e_depends):
        if r_name != e_name:
            return path
        return _find_difference(r_depend, e_depend, path + (r_name,))


def find_difference(result: Solution, expected: Solution) -> tuple | None:
    return _find_difference(result, expected, tuple())
