"""Module for base metadata implementation."""

from .attributes import BaseAttributeMetadata, collect_attributes
from .base import BaseMetadata, collect_metadata
from .qualifiers import BaseQualifierMetadata, collect_qualifiers
from .set import MetadataSet

__all__ = [
    "BaseMetadata",
    "collect_metadata",
    "BaseAttributeMetadata",
    "collect_attributes",
    "Name",
    "BaseQualifierMetadata",
    "collect_qualifiers",
    "DisallowSubclass",
    "MetadataSet",
]
