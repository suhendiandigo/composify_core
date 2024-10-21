from collections.abc import Sequence
from typing import Any, Iterable
from composify.metadata import MetadataSet
from composify.metadata.qualifiers import BaseQualifierMetadata

class Qualifiers:
    def __new__(items: Iterable[Any]): ...
    def __hash__(self): ...
    def __repr__(self): ...
    def qualify(self, attributes: MetadataSet) -> bool: ...


class TypeInfo:
    type_name: str
    type_module: str
    type_hash: int
    inner_type: type
    attributes: MetadataSet
    qualifiers: Qualifiers

    def __new__(type_info: type, metadata: Sequence[Any]): ...

    @staticmethod
    def parse(any_type: Any) -> "TypeInfo": ...

    def __hash__(self): ...
    def __repr__(self): ...
