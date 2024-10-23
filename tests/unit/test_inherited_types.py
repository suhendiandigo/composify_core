from dataclasses import dataclass
from typing import Annotated

import pytest

from composify.core.solutions import SolveSpecificity
from composify.errors import NoSolutionError, SolveFailureError
from composify.rules import rule
from tests.utils import create_rule_solver, solution


@dataclass(frozen=True)
class A:
    value: int


@dataclass(frozen=True)
class B:
    value: int


@dataclass(frozen=True)
class C(A):
    value: int


@rule
def create_a() -> A:
    return A(10)


@rule
def create_c() -> C:
    return C(100)


@pytest.mark.asyncio_cooperative
async def test_subclasses(compare_solutions):
    resolver = create_rule_solver(create_c)

    compare_solutions(
        resolver.solve_for(A),
        [solution(create_c)],
    )

    with pytest.raises(SolveFailureError) as exc:
        resolver.solve_for(B)
    assert exc.value.contains(NoSolutionError)

    compare_solutions(
        resolver.solve_for(C),
        [solution(create_c)],
    )


@pytest.mark.asyncio_cooperative
async def test_disallowed_subclasses(compare_solutions):
    resolver = create_rule_solver(create_c)

    with pytest.raises(SolveFailureError) as exc:
        resolver.solve_for(Annotated[A, SolveSpecificity.Exact])
    assert exc.value.contains(NoSolutionError)

    with pytest.raises(SolveFailureError) as exc:
        resolver.solve_for(B)
    assert exc.value.contains(NoSolutionError)

    compare_solutions(
        resolver.solve_for(C),
        [solution(create_c)],
    )


@pytest.mark.asyncio_cooperative
async def test_allowed_subclasses(compare_solutions):
    resolver = create_rule_solver(create_c)

    compare_solutions(
        resolver.solve_for(Annotated[A, SolveSpecificity.AllowSubclass]),
        [solution(create_c)],
    )

    with pytest.raises(SolveFailureError) as exc:
        resolver.solve_for(B)
    assert exc.value.contains(NoSolutionError)

    compare_solutions(
        resolver.solve_for(C),
        [solution(create_c)],
    )
