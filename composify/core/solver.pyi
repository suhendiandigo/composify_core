from collections.abc import Sequence
from typing import Any

from composify.core.registry import RuleRegistry
from composify.core.solutions import Solution


class Solver:
    def __new__(rules: RuleRegistry): ...
    def solve_for(self, type: Any) -> Sequence[Solution]: ...


class SolvingError: ...
