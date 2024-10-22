"""Base implementation for all annotated metadata."""

from functools import partial
from typing import Any, TypeVar

from composify.core.metadata import MetadataSet


class BaseMetadata:
    """Base class for all annotated metadata."""

    __slots__ = ()


M = TypeVar("M", bound=BaseMetadata)
T = TypeVar("T")


def _is_instance(type_: type, instance: Any) -> bool:
    return isinstance(instance, type_)


S = TypeVar("S", bound=MetadataSet)


def _collect_metadata(
    type_: type,
    metadata_type: type,
    set_type: type[S],
) -> S:
    vals: tuple[Any, ...] = getattr(type_, "__metadata__", ())
    if not vals:
        return set_type(vals)
    return set_type(filter(partial(_is_instance, metadata_type), vals))


def collect_metadata(type_: Any) -> MetadataSet:
    """Collect all annotated metadata that inherits BaseMetadata class as a frozenset."""
    return _collect_metadata(type_, BaseMetadata, MetadataSet)
