"""Implementation of Qualifiers."""

from abc import ABC, abstractmethod
from typing import Any

from .base import BaseMetadata, MetadataSet, _collect_metadata
from .attributes import AttributeSet


class BaseQualifierMetadata(BaseMetadata, ABC):
    """Base class for all qualifiers."""

    @abstractmethod
    def qualify(self, attributes: AttributeSet) -> bool:
        pass


class QualifierSet(MetadataSet):
    """A frozenset of Qualifiers."""

    def qualify(self, attributes: AttributeSet) -> bool:
        return all(qualifier.qualify(attributes) for qualifier in self._mapping.values())


def collect_qualifiers(type_: Any) -> QualifierSet:
    """Collect all BaseQualifierMetadata class.

    Args:
        type_ (Any): The type annotated with qualifiers.

    Returns:
        QualifierSet: Set of qualifiers.
    """
    return _collect_metadata(type_, BaseQualifierMetadata, QualifierSet)
