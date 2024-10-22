"""Implementation of Qualifiers."""

from typing import Protocol
from composify.core.metadata import MetadataSet

__all__ = (
    "MetadataSet", "Qualifier"
)


class Qualifier(Protocol):
    """Base class for all qualifiers."""

    def qualify(self, attributes: MetadataSet) -> bool:
        raise NotImplementedError()
