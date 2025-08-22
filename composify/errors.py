"""Module containing errors classes."""

from collections.abc import Iterable, Sequence
from typing import TypeAlias

from composify.core import Solution, TypeInfo


class InvalidTypeAnnotation(TypeError):
    """Raised for invalid type annotation."""

    pass


class MissingReturnTypeAnnotation(InvalidTypeAnnotation):
    """Raised when type annotation for a return value is missing."""

    pass


class MissingParameterTypeAnnotation(InvalidTypeAnnotation):
    """Raised when type annotation for a parameter is missing."""

    pass


Trace: TypeAlias = tuple[str, TypeInfo]
Traces: TypeAlias = Sequence[Trace]


def _format_trace(trace: Trace) -> str:
    return f"({trace[0]}: {trace[1]})"


def _format_traces(traces: Traces) -> str:
    steps = [str(traces[0][1]), *(_format_trace(trace) for trace in traces[1:])]
    return " -> ".join(steps)


class SolvingError(Exception):
    """Solving related errors."""

    pass


class SolveFailureError(SolvingError):
    """Raised when no solutions are found."""

    def __init__(self, errors: Iterable[SolvingError]) -> None:
        error_strings = tuple(
            f"- {_format_traces(error.traces)}: {error}"
            if isinstance(error, TracedSolvingError)
            else f"- {error}"
            for error in errors
        )
        error_string = "\n".join(error_strings)
        super().__init__(f"Solving failure:\n{error_string}")
        self.errors = errors

    def contains(self, exc_type: type[SolvingError]) -> bool:
        """Checks if any exception raised was of a specific type.

        Args:
            exc_type (type[SolvingError]): The exc type to find.

        Returns:
            bool: True if an exception of type exc_type exists; otherwise False.
        """
        for error in self.errors:
            if isinstance(error, exc_type):
                return True
        return False


class TracedSolvingError(SolvingError):
    """Raised when an error ocurred while solving containing the
    steps.
    """

    def __init__(self, traces: Traces, msg: str) -> None:
        super().__init__(msg)
        self.traces = traces


class NoSolutionError(TracedSolvingError):
    """Raised when there is no available solution."""

    def __init__(self, traces: Traces) -> None:
        super().__init__(traces, "Unable to find solution.")


class CyclicDependencyError(TracedSolvingError):
    """Raised when a cyclic dependency occurred in the dependency graph."""

    def __init__(self, traces: Traces) -> None:
        super().__init__(traces, "Encountered cyclic dependency.")


class NotExclusiveError(TracedSolvingError):
    """Raised when a dependency contains multiple solution in Exclusive cardinality."""

    def __init__(self, solutions: Sequence[Solution], traces: Traces) -> None:
        self.solutions = solutions
        super().__init__(
            traces,
            f"Found multiple solutions: {', '.join(str(solution) for solution in solutions)}",
        )


class BuilderError(Exception):
    """Base class for all Builder related errors."""

    pass


class AsyncSolutionError(BuilderError):
    """Raised when trying to build async solution using sync Builder."""

    pass
