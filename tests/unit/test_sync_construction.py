from dataclasses import dataclass

import pytest

from composify.builder import Builder
from composify.errors import AsyncSolutionError
from composify.rules import rule
from tests.utils import ExecutionCounter, solution, static


@dataclass(frozen=True)
class Value:
    value: int


@rule
def double(param: Value) -> Value:
    return Value(param.value * 2)


@rule
def quintuple(param: Value) -> Value:
    return Value(param.value * 5)


@rule
def squared(param: Value) -> Value:
    return Value(param.value**2)


def test_construct():
    plan = solution(
        double,
        param=static(Value(5)),
    )
    builder = Builder()
    result = builder.from_solution(plan)
    assert result == Value(10)


@rule
async def async_value() -> Value:
    return Value(1)


def test_construct_async():
    builder = Builder()

    plan = solution(
        async_value,
    )
    with pytest.raises(AsyncSolutionError):
        builder.from_solution(plan)

    plan = solution(double, param=solution(async_value))
    with pytest.raises(AsyncSolutionError):
        builder.from_solution(plan)


def test_cached_construct():
    counter = ExecutionCounter()

    double_ = counter(double)
    quintuple_ = counter(quintuple)
    squared_ = counter(squared)

    s1 = solution(
        double_,
        param=static(Value(5)),
    )
    s2 = solution(
        double_,
        param=static(Value(5)),
    )

    assert s1 == s2

    testcases = {
        solution(
            double_,
            param=static(Value(5)),
        ): 10,
        solution(
            double_,
            param=static(Value(5)),
        ): 10,
        solution(
            quintuple_,
            param=solution(
                double_,
                param=static(Value(5)),
            ),
        ): 50,
        solution(
            double_,
            param=solution(
                quintuple_,
                param=static(Value(5)),
            ),
        ): 50,
        solution(
            squared_,
            param=solution(
                double_,
                param=static(Value(5)),
            ),
        ): 100,
    }
    assert len(testcases) == 4

    builder = Builder()

    plans, expected_results = zip(*testcases.items())

    results = tuple(builder.from_solution(plan) for plan in plans)
    for result, expected_result in zip(results, expected_results):
        assert isinstance(result, Value)
        assert result.value == expected_result

    assert counter.execution == 5
