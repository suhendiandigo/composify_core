from dataclasses import dataclass
from typing import Annotated

import pytest

from composify.core import SolveCardinality
from composify.errors import NoSolutionError, SolveFailureError
from composify.rules import collect_rules, rule
from tests.utils import create_rule_solver, solution


@dataclass(frozen=True)
class A:
    value: int


@rule
def create_a() -> A:
    return A(100)


rules_1 = collect_rules()


@rule
def create_special() -> Annotated[A, "special"]:
    return A(10)


rules_2 = collect_rules()


@pytest.mark.asyncio_cooperative
async def test_no_named():
    solver = create_rule_solver(*rules_1)

    with pytest.raises(SolveFailureError) as exc:
        solver.solve_for(Annotated[A, "special"])
    assert exc.value.contains(NoSolutionError)


@pytest.mark.asyncio_cooperative
async def test_multiple_with_named(compare_solutions):
    solver = create_rule_solver(*rules_2)

    compare_solutions(
        solver.solve_for(Annotated[A, "special"]),
        [
            solution(create_special),
        ],
    )

    compare_solutions(
        solver.solve_for(Annotated[A, SolveCardinality.Exhaustive]),
        [
            solution(create_a),
            solution(create_special),
        ],
    )
