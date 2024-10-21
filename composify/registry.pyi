from collections.abc import Sequence
from composify.rules import Rule


class RuleRegistry:
    def add_rule(self, rule: Rule) -> None: ...
    def get_rules(self, type_info: type) -> Sequence[Rule]: ...

