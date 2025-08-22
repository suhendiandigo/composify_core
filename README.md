**Composify** is a Python framework designed to simplify the development of applications. By using **rule declarations** and **dependency injection**, Composify enables clean application structures.

Components and their relationships are declared upfront via the `@rule` decorator. The framework automatically builds a dependency graph and instantiates only the necessary components at runtime, handling dependency injection for you.

### Development

Usage of `uv` is recommended. Here are the steps to run the tests.

```
$ uv sync
$ uv run maturin develop
$ uv run pytest
```
