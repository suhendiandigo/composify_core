"""Implementation of Qualifiers."""

from typing import Protocol

from composify.core import MetadataSet

__all__ = ("MetadataSet", "Qualifier")


class Qualifier(Protocol):
    """Protocol for qualifier implementation.
    This allows for granular customization of how rules are chosen.
    """

    def qualify(self, attributes: MetadataSet) -> bool:
        """Returns true if the set of attributes is qualified."""
        raise NotImplementedError()
