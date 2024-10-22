from collections.abc import Callable, Iterator, Mapping

from composify.core import TypeInfo

class Dependency:
    name: str
    typing: TypeInfo

    def __hash__(self): ...

class Dependencies:
    def __iter__(self) -> Iterator[Dependency]: ...
    def __hash__(self): ...

class Rule:
    function: Callable
    canonical_name: str
    output_type: TypeInfo
    dependencies: Dependencies
    priority: int
    is_async: bool

    def __new__(
        function: Callable,
        canonical_name: str,
        output_type: type,
        dependencies: Mapping[str, type],
        priority: int,
        is_async: bool,
    ): ...
    def __hash__(self): ...
