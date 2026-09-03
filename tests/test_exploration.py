"""Tests for attribute_exploration with Python callback."""
import pytest
import odis


def test_always_accept_equals_canonical_basis(living_beings_path):
    ctx1 = odis.FormalContext.from_file(living_beings_path)
    ctx2 = odis.FormalContext.from_file(living_beings_path)

    # always-accept callback: return True for every implication
    def always_accept(premise, conclusion):
        return True

    exploration_basis = ctx1.attribute_exploration(always_accept)
    canonical = ctx2.canonical_basis()

    assert isinstance(exploration_basis, odis.ImplicationList)
    assert len(exploration_basis) == len(canonical)

    # Check set equivalence via to_python
    expl_set = {
        (frozenset(p), frozenset(c)) for p, c in exploration_basis.to_python()
    }
    can_set = {
        (frozenset(p), frozenset(c)) for p, c in canonical.to_python()
    }
    assert expl_set == can_set


def test_counterexample_adds_object():
    """A callback that rejects with a counterexample must add the object to ctx."""
    ctx = odis.FormalContext.from_dict({
        "robin": {"flies", "has_wings"},
        "eagle": {"flies", "has_wings"},
    })
    initial_count = len(ctx.objects)

    call_count = [0]

    def counter_callback(premise, conclusion):
        if call_count[0] == 0:
            call_count[0] += 1
            # Reject first implication with a counterexample
            return ("penguin", {"has_wings"})
        return True  # accept all subsequent

    ctx.attribute_exploration(counter_callback)
    assert "penguin" in ctx.objects
    assert len(ctx.objects) > initial_count


def test_callback_exception_propagates(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)

    def raising_callback(premise, conclusion):
        raise ValueError("callback error from test")

    with pytest.raises(ValueError, match="callback error from test"):
        ctx.attribute_exploration(raising_callback)


def test_exploration_exempt_from_lazy_guard(living_beings_path):
    """attribute_exploration should run even when a lazy generator is alive."""
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = ctx.concepts(lazy=True)
    # Keep generator alive — consume one item
    c = next(gen)
    assert isinstance(c, odis.Concept)

    # attribute_exploration should NOT raise RuntimeError due to the live generator
    # exploration is exempt from the lazy mutation guard)
    def always_accept(p, q):
        return True

    # This should succeed without RuntimeError
    result = ctx.attribute_exploration(always_accept)
    assert isinstance(result, odis.ImplicationList)


def test_callback_receives_labelsets(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    received = []

    def capture_callback(premise, conclusion):
        received.append((type(premise).__name__, type(conclusion).__name__))
        return True

    ctx.attribute_exploration(capture_callback)

    assert len(received) > 0, "callback was never called"
    for prem_type, conc_type in received:
        assert prem_type == "LabelSet", f"premise type was {prem_type}"
        assert conc_type == "LabelSet", f"conclusion type was {conc_type}"
