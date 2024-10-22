from collections.abc import Sequence
from enum import Enum, auto

from composify.rules import Rule


class SolveSpecificity(Enum):
    Exact = auto()
    AllowSubclass = auto()
    AllowSuperclass = auto()


class SolveCardinality(Enum):
    Exhaustive = auto()
    Single = auto()
    Exclusive = auto()


class SolveParameter:
    def __new__(cls, specificity: SolveSpecificity, cardinality: SolveCardinality): ...
    @property
    def specificity(self) -> SolveSpecificity: ...
    @property
    def cardinality(self) -> SolveCardinality: ...


class SolutionArg:
    @property
    def name(self) -> str: ...
    @property
    def solution(self) -> Solution: ...


class SolutionArgsCollection(Sequence[SolutionArg]):
    pass

class Solution:
    
    @property
    def rule(self) -> Rule: ...
    @property
    def args(self) -> SolutionArgsCollection: ...
