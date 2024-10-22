from typing import Any

from composify.core.solutions import Solution
from composify.rules import as_rule, static_rule, wraps_rule


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
