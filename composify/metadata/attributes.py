"""Implementation of Attributes."""

from typing import Any
from .base import BaseMetadata, _collect_metadata

from .set import MetadataSet


class BaseAttributeMetadata(BaseMetadata):
    """Base class for all attributes."""

    pass


class AttributeSet(MetadataSet):
    """A frozenset of Attributes."""

    pass


def collect_attributes(type_: Any) -> AttributeSet:
    """Collect all BaseAttributeMetadata class.

    Args:
        type_ (Any): The type annotated with attributes.

    Returns:
        AttributeSet: Set of attributes.
    """
    return _collect_metadata(type_, BaseAttributeMetadata, AttributeSet)
