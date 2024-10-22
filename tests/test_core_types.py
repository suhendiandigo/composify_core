from dataclasses import dataclass
from typing import Annotated

from composify.core import TypeInfo
from composify.metadata.attributes import AttributeSet, BaseAttributeMetadata
from composify.metadata.qualifiers import BaseQualifierMetadata
from composify.core.registry import RuleRegistry
from composify.rules import Rule
from composify.core.solutions import SolveCardinality, SolveSpecificity


@dataclass(frozen=True)
class NameAttr(BaseAttributeMetadata):
    name: str


def example_fn():
    pass


@dataclass(frozen=True)
class NameQualifier(BaseQualifierMetadata):
    name: str

    def qualify(self, attributes: AttributeSet) -> bool:
        if attr := attributes.get(NameAttr):
            return attr.name == self.name
        return False


def test_registry():
    reg = RuleRegistry()
    r = Rule(example_fn, "test", str, {"in1": str, "in2": str}, 3, False, is_optional=False)
    reg.add_rule(r)
    assert hash((r,)) == hash(reg.get_rules(r.output_type))
    assert hash(r.output_type) == hash(reg.get_rules(r.output_type)[0].output_type)


def test_type_info_qualifier():
    stored = TypeInfo.parse(Annotated[str, NameAttr("test2")])
    to_solve = TypeInfo.parse(Annotated[str, NameQualifier("test2")])
    assert to_solve.qualifiers.qualify(stored.attributes)


def test_solve_parameter():
    stored = TypeInfo.parse(Annotated[str, NameAttr("test2")])
    to_solve = TypeInfo.parse(Annotated[str, NameQualifier("test2"), SolveCardinality.Exhaustive, SolveSpecificity.AllowSuperclass])
    assert to_solve.qualifiers.qualify(stored.attributes)
    assert to_solve.solve_parameter.cardinality == SolveCardinality.Exhaustive
    assert to_solve.solve_parameter.specificity == SolveSpecificity.AllowSuperclass


def test_registry_qualifier():
    reg = RuleRegistry()
    r1 = Rule(example_fn, "test", str, {"in1": str, "in2": str}, 3, False, is_optional=False)
    r2 = Rule(example_fn, "test", Annotated[str, NameAttr("test2")], {"in1": str, "in2": str}, 3, False, is_optional=False)
    reg.add_rule(r1)
    reg.add_rule(r2)

    assert hash((r2,)) == hash(reg.get_rules(Annotated[str, NameQualifier("test2")]))
