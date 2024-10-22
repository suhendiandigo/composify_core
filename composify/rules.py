"""This modules contains the backbone @rule decorator.

Example:
    from composify import collect_rules, rule

    @rule
    def create_default() -> int:
        return 1

    rules = collect_rules()

    print(len(rules) == 1)
    #> True

"""

import asyncio
import inspect
from collections.abc import Iterable, Mapping
from functools import partial
from types import FrameType, ModuleType
from typing import (
    Annotated,
    Any,
    Callable,
    ParamSpec,
    TypeVar,
    get_type_hints,
)

from composify.errors import InvalidTypeAnnotation, MissingParameterTypeAnnotation, MissingReturnTypeAnnotation
from composify.core.rules import Rule
from composify.qualifiers import Qualifier

__all__ = ("rule", "as_rule")

def ensure_type_annotation(
    *,
    type_annotation: Any,
    name: str,
    raise_type: type[InvalidTypeAnnotation] = InvalidTypeAnnotation,
) -> Any:
    if type_annotation is None:
        raise raise_type(f"{name} is missing a type annotation.")
    return type_annotation


def resolve_type_name(value: Any) -> str:
    """Resolve qualified name of a value."""
    return f"{value.__module__}.{value.__qualname__}".replace(".<locals>", "")


T = TypeVar("T")
P = ParamSpec("P")

RULE_ATTR = "__rule__"

class ConstructRuleSet(tuple[Rule, ...]):
    pass


def _add_qualifiers(
    type_: Any, qualifiers: Iterable[Qualifier] | None
) -> Any:
    if qualifiers is not None:
        for qualifier in qualifiers:
            type_ = Annotated[type_, qualifier]
    return type_


def _get_init_func(cls: type):
    if cls.__init__ == object.__init__:  # type: ignore[misc]
        func = cls
        func_params = []
    else:
        func = cls.__init__  # type: ignore[misc]
        func_params = list(inspect.signature(func).parameters)[1:]
    return func, func_params


def attach_rule(value: Any, rule: Rule | ConstructRuleSet) -> None:
    """Attach a rule to an object that is collectible via collect_rules().
    To be used with custom rule decorators.

    Args:
        value (Any): The object to attach the rule to.
        rule (ConstructRule | ConstructRuleSet): The rule to attach.
    """
    setattr(
        value,
        RULE_ATTR,
        rule,
    )


def _rule_decorator(
    decorated: Callable,
    *,
    priority: int,
    name: str | None = None,
    dependency_qualifiers: Iterable[Qualifier] | None = None,
    return_type: type | None = None,
    is_optional: bool | None = None,
) -> Any:
    if inspect.isclass(decorated):
        func, func_params = _get_init_func(decorated)
    else:
        func = decorated
        func_params = list(inspect.signature(func).parameters)
    name = name or f"{func.__module__}:{func.__name__}"
    func_id = f"@rule {name}"
    type_hints = get_type_hints(func, include_extras=True)
    return_type = return_type or (
        decorated if inspect.isclass(decorated) else type_hints.get("return")
    )
    return_type_info = ensure_type_annotation(
        type_annotation=return_type,
        name=f"{func_id} return",
        raise_type=MissingReturnTypeAnnotation,
    )

    parameter_types: Mapping[str, Any] = dict(
        (
            parameter,
            _add_qualifiers(
                ensure_type_annotation(
                    type_annotation=type_hints.get(parameter),
                    name=f"{func_id} parameter {parameter}",
                    raise_type=MissingParameterTypeAnnotation,
                ),
                dependency_qualifiers,
            ),
        )
        for parameter in func_params
    )
    effective_name = resolve_type_name(decorated)

    rule = Rule(
        decorated,
        canonical_name=effective_name,
        output_type=return_type_info,
        dependencies=parameter_types,
        priority=priority,
        is_async=asyncio.iscoroutinefunction(func),
    )
    attach_rule(decorated, rule)
    return decorated


def rule(
    f: Callable | None = None,
    /,
    *,
    priority: int = 0,
    name: str | None = None,
    dependency_qualifiers: Iterable[Qualifier] | None = None,
    return_type: type | None = None,
    is_optional: bool | None = None,
):
    """Marks a function or a class as a rule. Allowing collection via collect_rules().

    Args:
        f (RuleFunctionType | None, optional): The function or class to mark as a rule. Defaults to None.
        name (str | None, optional): Override the name of the rule if exists.
        priority (int, optional): The resolution priority. Higher value equals higher priority. Defaults to 0.
        dependency_qualifiers (Iterable[BaseQualifierMetadata] | None, optional): Add qualifiers to all dependencies. Defaults to None.
        return_type (type | None, optional): Override the return type of the rule.
        is_optional (bool | None, optional): Override the optionality of the rule.

    Returns:
        The input function or class.

    Raises:
        MissingReturnTypeAnnotation: Raised if the return type annotation is missing.
        MissingParameterTypeAnnotation: Raised if there are any missing type annotation from parameter.
        DuplicatedEntryError: Raised if there are duplicated rule.

    """
    if f is None:
        return partial(
            _rule_decorator,
            priority=priority,
            name=name,
            dependency_qualifiers=dependency_qualifiers,
            return_type=return_type,
            is_optional=is_optional,
        )
    return _rule_decorator(
        f,
        priority=priority,
        name=name,
        dependency_qualifiers=dependency_qualifiers,
        return_type=return_type,
        is_optional=is_optional,
    )


def as_rule(f: Any) -> Rule:
    """Returns the ConstructRule associated with the object.

    Args:
        f (Any): The object to cast as ConstructRule.

    Returns:
        ConstructRule | None: Returns the ConstructRule if the object has been marked with @rule; otherwise None.
    """
    if isinstance(f, Rule):
        return f
    r = getattr(f, RULE_ATTR, None)
    if r is None:
        raise TypeError(f"{f} is not a rule.")
    return r


def _extract_rules(rule: Any):
    if isinstance(rule, Rule):
        yield rule
    elif isinstance(rule, ConstructRuleSet):
        for r in rule:
            yield from _extract_rules(r)
    else:
        if not callable(rule):
            return
        rule = getattr(rule, RULE_ATTR, None)
        yield from _extract_rules(rule)


def collect_rules(
    *namespaces: ModuleType | Mapping[str, Any],
) -> Iterable[Rule]:
    if not namespaces:
        currentframe = inspect.currentframe()
        assert isinstance(currentframe, FrameType)
        caller_frame = currentframe.f_back
        assert isinstance(caller_frame, FrameType)

        global_items = caller_frame.f_globals
        namespaces = (global_items,)

    def iter_rules():
        for namespace in namespaces:
            mapping = (
                namespace.__dict__
                if isinstance(namespace, ModuleType)
                else namespace
            )
            for item in mapping.values():
                yield from _extract_rules(item)

    return list(iter_rules())
