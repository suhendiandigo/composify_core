from collections.abc import Sequence
from typing import Any, Iterable
from composify.metadata import MetadataSet
from composify.solutions import SolveParameter


class TypeInfo:
    type_name: str
    type_module: str
    type_hash: int
    inner_type: type
    attributes: MetadataSet
    qualifiers: Qualifiers
    solve_parameter: SolveParameter

    def __new__(type_info: type, metadata: Sequence[Any]): ...

    @staticmethod
    def parse(any_type: Any) -> "TypeInfo": ...

    def __hash__(self): ...
    def __repr__(self): ...
