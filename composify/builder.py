"""Implementation of Builder and AsyncBuilder to build using solution."""

import asyncio
from concurrent.futures import ThreadPoolExecutor
from functools import partial
from typing import Any, TypeVar

from composify.core.solutions import Solution
from composify.errors import AsyncSolutionError

__all__ = [
    "AsyncBuilder",
    "Builder",
]


T = TypeVar("T")


class AsyncBuilder:
    """Build objects from solutions. Supports async solutions."""

    _cache: dict[Solution, asyncio.Task[Any]]

    def __init__(
        self,
        threadpool_executor: ThreadPoolExecutor | None = None,
    ) -> None:
        self._cache = {}
        self._threadpool_executor = threadpool_executor

    async def get_cached(self, solution: Solution) -> T | None:
        """Get cached value for a solution.

        Args:
            solution (Solution): The solution cache to find.

        Returns:
            T | None: The cached value if it exists; otherwise None.
        """
        cached = self._cache.get(solution, None)
        return await cached if cached else None

    async def from_solution(self, solution: Solution) -> T:
        """Build an object using a solution.

        Args:
            solution (Solution): A solution to base on.

        Returns:
            T: A built object.
        """
        task = self._cache.get(solution, None)
        if task is not None:
            return await task
        task = asyncio.Task(self._from_solution(solution))

        # We cache the coroutine instead of the result
        # This allows asynchronous requests to share the same coroutine
        self._cache[solution] = task

        value = await task

        return value

    async def _from_solution(self, solution: Solution) -> T:
        name_task_pairs = tuple(
            (arg.name, self.from_solution(arg.solution))
            for arg in solution.args
        )

        names = tuple(p[0] for p in name_task_pairs)
        tasks = tuple(p[1] for p in name_task_pairs)

        results = tuple(await asyncio.gather(*tasks))

        parameters = dict(zip(names, results, strict=True))

        if asyncio.iscoroutinefunction(solution.function):
            value = await solution(**parameters)
        elif self._threadpool_executor is not None:
            loop = asyncio.get_running_loop()
            value = await loop.run_in_executor(
                self._threadpool_executor,
                partial(solution.function, **parameters),  # type: ignore[arg-type]
            )
        else:
            value = solution.function(**parameters)

        return value


class Builder:
    """Builder objects from solutions. Does not support async solutions."""

    _cache: dict[Solution, Any]

    def __init__(
        self,
    ) -> None:
        self._cache = {}

    def from_solution(self, solution: Solution) -> T:
        """Build an object using a solution.

        Args:
            solution (Solution): A solution to base on.

        Raises:
            AsyncSolutionError: If the solution requires async loop.

        Returns:
            T: A built object.
        """
        if solution.is_async:
            raise AsyncSolutionError(
                f"Trying to build from async solution {solution}"
            )
        value = self._cache.get(solution, None)
        if value is not None:
            return value

        value = self._from_solution(solution)

        self._cache[solution] = value

        return value

    def _from_solution(self, solution: Solution) -> T:
        parameters = {
            arg.name: self.from_solution(arg.solution) for arg in solution.args
        }

        value = solution.function(**parameters)

        return value
