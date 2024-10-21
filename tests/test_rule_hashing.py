from composify_core.rules import Rule


def example_fn():
    pass


def test_hashing():
    r1 = Rule(example_fn, "test", str, {"in1": str, "in2": str}, 3, False, is_optional=False)
    r2 = Rule(example_fn, "test", str, {"in1": str, "in2": str}, 3, False, is_optional=False)
    
    assert hash(r1) == hash(r2), "Different hashes"
    
    assert hash(r1.dependencies) == hash(r2).dependencies
