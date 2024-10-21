from collections.abc import Sequence
from typing import Any
from composify.metadata.attributes import BaseAttributeMetadata
from composify.metadata.qualifiers import BaseQualifierMetadata

class TypeInfo:
    type_name: str
    type_module: str
    type_hash: int
    inner_type: type
    attributes: list[BaseAttributeMetadata]
    attribute_hash: int
    qualifiers: list[BaseQualifierMetadata]

    def __new__(type_info: type, metadata: Sequence[Any]): ...

    @staticmethod
    def parse(any_type: Any) -> "TypeInfo": ...

    def __hash__(self): ...
