from collections.abc import Iterable
from itertools import zip_longest

from pytest import fixture

from composify.core.solutions import Solution
from tests.utils import find_difference


def _select_bp(bp: Solution, path: tuple[str, ...]) -> Solution:
    if not path:
        return bp
    top, rest = path[0], path[1:]
    return _select_bp(
        next(iter(filter(lambda x: x[0] == top, bp.args)))[1], rest
    )


_COMPARED_ATTRS = ("constructor", "is_async", "dependencies")


def pytest_assertrepr_compare(op, left, right):
    if (
        isinstance(left, Solution)
        and isinstance(right, Solution)
        and op == "=="
    ):
        diff_path = find_difference(left, right)
        if diff_path is not None:
            result = ["solution instances:"]
            path_str = "->".join(("root",) + diff_path)
            result.append(f"   path: {path_str}")
            left_bp = _select_bp(left, diff_path)
            right_bp = _select_bp(right, diff_path)
            for attr in _COMPARED_ATTRS:
                left_val = getattr(left_bp, attr)
                right_val = getattr(right_bp, attr)
                if not left_val == right_val:
                    result.append(f"   {attr}: {left_val} != {right_val}")
            return result


def _compare_solutions(
    solutions: Iterable[Solution], expected_solutions: Iterable[Solution]
):
    solutions = list(solutions)
    expected_solutions = list(expected_solutions)
    assert len(solutions) == len(
        expected_solutions
    ), f"different plan len {len(solutions)} != {len(expected_solutions)}"
    for index, (plan, expected) in enumerate(
        zip_longest(solutions, expected_solutions)
    ):
        assert plan == expected, f"case {index}"


@fixture
def compare_solutions():
    return _compare_solutions
