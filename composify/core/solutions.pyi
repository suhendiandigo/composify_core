from collections.abc import Callable, Mapping, Sequence
from enum import Enum, auto

from composify.core import TypeInfo
from composify.rules import Rule

class SolveSpecificity(Enum):
    """Determine the specificity of the solutions' result types:
    - (=) Exact: Allow only for exact type. No superclasses or subclasses are allowed.
    - (+) AllowSubclass: Allow for solutions resulting in subclasses.
    - (-) AllowSuperclass: Allow for solutions resulting in superclasses.
    """

    Exact = auto()
    AllowSubclass = auto()
    AllowSuperclass = auto()

class SolveCardinality(Enum):
    """Determine the number of solutions to match when solving for a specific type:
    - (*) Exhaustive: Solve for all possible solution including all permutations of dependencies.
    - (1) Single: Solve for the first possible solution respecting the priority of rules, ignoring the rest of the solutions.
    - (x) Exclusive: Solve for a single possible solution. Raises error if there are multiple solutions including permutations of dependencies.
    """

    Exhaustive = auto()
    Single = auto()
    Exclusive = auto()

class SolveParameter:
    def __new__(
        cls, specificity: SolveSpecificity, cardinality: SolveCardinality
    ): ...
    @property
    def specificity(self) -> SolveSpecificity: ...
    @property
    def cardinality(self) -> SolveCardinality: ...

class SolutionArg:
    @property
    def name(self) -> str: ...
    @property
    def solution(self) -> Solution: ...
    def __hash__(self): ...

class SolutionArgsCollection(Sequence[SolutionArg]):
    def __hash__(self): ...

class Solution:
    def __new__(self, rule: Rule, args: Mapping[str, Solution] | None): ...
    @property
    def rule(self) -> Rule: ...
    @property
    def args(self) -> SolutionArgsCollection: ...
    @property
    def function(self) -> Callable[..., T]: ...
    @property
    def output_type(self) -> TypeInfo: ...
    @property
    def is_async(self) -> bool: ...
    def __hash__(self): ...
