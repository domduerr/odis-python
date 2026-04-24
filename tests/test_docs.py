"""Tests for every code example in odis-python/README.md.

One example_*() + test_*() pair per README code block.
example_*() functions are self-contained; test_*() functions assert at least
one observable property of the output.
"""
import os
import tempfile
from pathlib import Path

import pytest

import odis

# ---------------------------------------------------------------------------
# Shared test-data helper — mirrors conftest.py pattern
# ---------------------------------------------------------------------------

_TESTDATA = Path(__file__).parent.parent.parent / "odis" / "test_data"


def _cxt(name: str) -> str:
    """Return the absolute path to a test-data .cxt file."""
    return str(_TESTDATA / name)


# ---------------------------------------------------------------------------
# Quick Start
# ---------------------------------------------------------------------------


def example_quick_start():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    objects = ctx.objects
    attributes = ctx.attributes
    concepts = list(ctx.concepts())
    return ctx, objects, attributes, concepts


def test_quick_start():
    ctx, objects, attributes, concepts = example_quick_start()
    assert isinstance(ctx, odis.FormalContext)
    assert len(objects) == 8
    assert len(attributes) == 9
    assert len(concepts) == 19


# ---------------------------------------------------------------------------
# Construction — empty constructor
# ---------------------------------------------------------------------------


def example_construction_empty():
    from odis import FormalContext
    ctx = FormalContext()
    return ctx


def test_construction_empty():
    ctx = example_construction_empty()
    assert ctx.shape == (0, 0)
    assert ctx.objects == []
    assert ctx.attributes == []


# ---------------------------------------------------------------------------
# Construction — from_file
# ---------------------------------------------------------------------------


def example_construction_from_file():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    return ctx


def test_construction_from_file():
    ctx = example_construction_from_file()
    assert isinstance(ctx, odis.FormalContext)
    assert len(ctx.objects) == 8
    assert len(ctx.attributes) == 9


# ---------------------------------------------------------------------------
# Construction — from_dict
# ---------------------------------------------------------------------------


def example_construction_from_dict():
    from odis import FormalContext
    ctx = FormalContext.from_dict({
        "cat":  {"has_legs", "has_fur", "can_move"},
        "fish": {"lives_in_water", "can_move"},
        "fern": {"needs_chlorophyll"},
    })
    return ctx


def test_construction_from_dict():
    ctx = example_construction_from_dict()
    assert "cat" in ctx
    assert "fern" in ctx
    assert len(ctx.objects) == 3
    assert "can_move" in ctx.attributes


# ---------------------------------------------------------------------------
# Introspection
# ---------------------------------------------------------------------------


def example_introspection():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    n_objects, n_attributes = ctx.shape
    n = len(ctx)
    objects = ctx.objects
    attributes = ctx.attributes
    has_frog = "frog" in ctx
    r = repr(ctx)
    return n_objects, n_attributes, n, objects, attributes, has_frog, r


def test_introspection():
    n_objects, n_attributes, n, objects, attributes, has_frog, r = example_introspection()
    assert n_objects == 8
    assert n_attributes == 9
    assert n == 8
    assert "frog" in objects
    assert has_frog is True
    assert isinstance(r, str)


# ---------------------------------------------------------------------------
# Incidence access
# ---------------------------------------------------------------------------


def example_incidence_access():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    val_true = ctx["frog", "lives in water"]
    val_false = ctx["frog", "breast feeds"]
    ctx["frog", "lives in water"] = False
    after_false = ctx["frog", "lives in water"]
    ctx["frog", "lives in water"] = True
    after_true = ctx["frog", "lives in water"]
    return val_true, val_false, after_false, after_true


def test_incidence_access():
    val_true, val_false, after_false, after_true = example_incidence_access()
    assert val_true is True
    assert val_false is False
    assert after_false is False
    assert after_true is True


# ---------------------------------------------------------------------------
# Mutation
# ---------------------------------------------------------------------------


def example_mutation():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    ctx.add_object("whale")
    has_whale = "whale" in ctx
    ctx.add_object("shark", {"needs water to live", "can move"})
    ctx.add_attribute("is_endangered")
    has_attr = "is_endangered" in ctx.attributes
    ctx.remove_object("whale")
    still_has_whale = "whale" in ctx
    ctx.remove_attribute("is_endangered")
    ctx.rename_object("frog", "toad")
    renamed_obj = "toad" in ctx
    ctx.rename_object("toad", "frog")  # restore
    return has_whale, has_attr, still_has_whale, renamed_obj


def test_mutation():
    has_whale, has_attr, still_has_whale, renamed_obj = example_mutation()
    assert has_whale is True
    assert has_attr is True
    assert still_has_whale is False
    assert renamed_obj is True


# ---------------------------------------------------------------------------
# Serialisation
# ---------------------------------------------------------------------------


def example_serialisation():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    with tempfile.NamedTemporaryFile(suffix=".cxt", delete=False) as f:
        tmp_path = f.name
    ctx.to_file(tmp_path)
    ctx2 = FormalContext.from_file(tmp_path)
    os.unlink(tmp_path)

    copy = ctx.copy()
    copy.add_object("clone_only")
    original_has_it = "clone_only" in ctx.objects
    return ctx2, original_has_it


def test_serialisation():
    ctx2, original_has_it = example_serialisation()
    assert isinstance(ctx2, odis.FormalContext)
    assert len(ctx2.objects) == 8
    assert original_has_it is False


# ---------------------------------------------------------------------------
# Derivation operators
# ---------------------------------------------------------------------------


def example_derivation_operators():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    extent = ctx.extent(["needs water to live", "can move"])
    intent = ctx.intent(["fish", "leech", "bream"])
    hull = ctx.attribute_hull(["needs water to live"])
    ohull = ctx.object_hull(["frog"])
    neighbor = ctx.upper_neighbor(["frog"])
    return extent, intent, hull, ohull, neighbor


def test_derivation_operators():
    extent, intent, hull, ohull, neighbor = example_derivation_operators()
    assert isinstance(extent, odis.LabelSet)
    assert isinstance(intent, odis.LabelSet)
    assert isinstance(hull, odis.LabelSet)
    assert isinstance(ohull, odis.LabelSet)
    assert isinstance(neighbor, odis.LabelSet)
    # Extent of objects sharing "needs water to live" and "can move"
    extent_list = list(extent)
    assert len(extent_list) >= 1


# ---------------------------------------------------------------------------
# Drawing shortcut
# ---------------------------------------------------------------------------


def example_drawing_shortcut():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    svg_str = ctx.draw_svg("dimdraw", width=800, height=600)
    drawing = ctx.draw("dimdraw")
    return svg_str, drawing


def test_drawing_shortcut():
    svg_str, drawing = example_drawing_shortcut()
    assert isinstance(svg_str, str)
    assert svg_str.startswith("<svg")
    assert drawing is not None


# ---------------------------------------------------------------------------
# Concepts — eager
# ---------------------------------------------------------------------------


def example_concepts_eager():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    concepts = ctx.concepts()
    count = len(concepts)
    first = concepts[0]
    extents = [list(ext) for ext, intent in concepts]
    return count, first, extents


def test_concepts_eager():
    count, first, extents = example_concepts_eager()
    assert count == 19
    assert isinstance(first, odis.Concept)
    assert isinstance(first.extent, odis.LabelSet)
    assert isinstance(first.intent, odis.LabelSet)
    assert len(extents) == 19


# ---------------------------------------------------------------------------
# Implications
# ---------------------------------------------------------------------------


def example_implications():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    basis = ctx.canonical_basis()
    n = len(basis)
    imp = basis[0]
    premise_list = list(imp.premise)
    conclusion_list = list(imp.conclusion)
    basis_opt = ctx.canonical_basis_optimised()
    return n, premise_list, conclusion_list, len(basis_opt)


def test_implications():
    n, premise_list, conclusion_list, n_opt = example_implications()
    assert n > 0
    assert isinstance(premise_list, list)
    assert isinstance(conclusion_list, list)
    assert n == n_opt  # both variants give the same basis size


# ---------------------------------------------------------------------------
# next_preclosure
# ---------------------------------------------------------------------------


def example_next_preclosure():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    basis = ctx.canonical_basis()
    n_attrs = len(ctx.attributes)
    pseudo_intents = []
    current = frozenset()
    while len(current) < n_attrs:
        nxt = ctx.next_preclosure(basis, current)
        if len(nxt) == n_attrs:
            break
        pseudo_intents.append(list(nxt))
        current = nxt
    return pseudo_intents, n_attrs


def test_next_preclosure():
    pseudo_intents, n_attrs = example_next_preclosure()
    assert isinstance(pseudo_intents, list)
    # next_preclosure iterates all closed sets in lectic order (both pseudo-intents
    # and concept intents), so the count is >= the canonical basis size
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    n_basis = len(ctx.canonical_basis())
    assert len(pseudo_intents) >= n_basis
    assert all(isinstance(ls, list) for ls in pseudo_intents)


# ---------------------------------------------------------------------------
# Attribute Exploration — accepting and rejecting paths
# ---------------------------------------------------------------------------


def example_attribute_exploration():
    from odis import FormalContext
    ctx = FormalContext.from_dict({
        "robin": {"flies", "has_wings"},
        "eagle": {"flies", "has_wings"},
    })
    call_log = []

    def oracle(premise, conclusion):
        call_log.append((list(premise), list(conclusion)))
        return True  # accept everything (produces same basis as canonical_basis)

    basis = ctx.attribute_exploration(oracle)
    return basis, call_log


def test_attribute_exploration():
    basis, call_log = example_attribute_exploration()
    assert isinstance(basis, odis.ImplicationList)
    assert len(call_log) > 0  # oracle was called at least once


def example_attribute_exploration_reject():
    """Demonstrate the reject path: counterexample adds an object to the context."""
    from odis import FormalContext
    ctx = FormalContext.from_dict({
        "robin": {"flies", "has_wings"},
        "eagle": {"flies", "has_wings"},
    })
    calls = [0]

    def oracle(premise, conclusion):
        if calls[0] == 0:
            calls[0] += 1
            return ("penguin", {"has_wings"})  # reject with counterexample
        return True

    ctx.attribute_exploration(oracle)
    return ctx


def test_attribute_exploration_reject():
    ctx = example_attribute_exploration_reject()
    assert "penguin" in ctx.objects


# ---------------------------------------------------------------------------
# Drawing — SVG to file
# ---------------------------------------------------------------------------


def example_drawing_svg():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    svg = ctx.draw_svg("dimdraw", width=800, height=600)
    with tempfile.NamedTemporaryFile(suffix=".svg", delete=False, mode="w") as f:
        f.write(svg)
        tmp_path = f.name
    with open(tmp_path) as f:
        content = f.read()
    os.unlink(tmp_path)
    return svg, content


def test_drawing_svg():
    svg, content = example_drawing_svg()
    assert svg.startswith("<svg")
    assert "</svg>" in svg
    assert content == svg


# ---------------------------------------------------------------------------
# Drawing — Drawing object
# ---------------------------------------------------------------------------


def example_drawing_object():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    drawing = ctx.draw("dimdraw")
    if drawing is None:
        return None, None, None, None
    node_count = len(drawing.nodes)
    edges = drawing.edges
    coords = drawing.coordinates
    svg2 = drawing.to_svg(ctx, width=1200, height=800)
    return node_count, edges, coords, svg2


def test_drawing_object():
    node_count, edges, coords, svg2 = example_drawing_object()
    assert node_count is not None
    assert node_count > 0
    assert isinstance(edges, list)
    assert isinstance(coords, list)
    assert svg2.startswith("<svg")


# ---------------------------------------------------------------------------
# Iceberg Lattice (Titanic)
# ---------------------------------------------------------------------------


def example_iceberg():
    from odis import FormalContext, Titanic
    ctx = FormalContext.from_dict({
        "a": {"x", "y", "z"},
        "b": {"x", "y"},
        "c": {"x", "z"},
        "d": {"y", "z"},
        "e": {"x"},
    })
    iceberg = Titanic()
    top_concepts = iceberg.enumerate(ctx, min_support=2)
    results = [(list(c.extent), list(c.intent)) for c in top_concepts]
    return len(top_concepts), results


def test_iceberg():
    n, results = example_iceberg()
    assert n >= 1  # at least one concept with support ≥ 2
    # All returned extents must have ≥ 2 objects
    for extent_list, _ in results:
        assert len(extent_list) >= 2


# ---------------------------------------------------------------------------
# LabelSet
# ---------------------------------------------------------------------------


def example_labelset():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    intent = ctx.intent(["fish", "leech"])
    can_move_present = "can move" in intent
    labels = [attr for attr in intent]
    as_list = list(intent)
    as_set = set(intent)
    return intent, can_move_present, labels, as_list, as_set


def test_labelset():
    intent, can_move_present, labels, as_list, as_set = example_labelset()
    assert isinstance(intent, odis.LabelSet)
    assert isinstance(as_list, list)
    assert isinstance(as_set, set)
    assert len(as_list) == len(as_set)
    assert "needs water to live" in as_set


# ---------------------------------------------------------------------------
# Lazy Generators & Mutation Guard
# ---------------------------------------------------------------------------


def example_lazy_mutation_guard():
    from odis import FormalContext
    ctx = FormalContext.from_file(_cxt("living_beings_and_water.cxt"))
    gen = ctx.concepts(lazy=True)
    first = next(gen)
    first_extent = list(first.extent)
    got_runtime_error = False
    try:
        ctx.add_attribute("new_attr")
    except RuntimeError:
        got_runtime_error = True
    del gen
    ctx.add_attribute("new_attr")
    mutation_succeeded = "new_attr" in ctx.attributes
    return first_extent, got_runtime_error, mutation_succeeded


def test_lazy_mutation_guard():
    first_extent, got_runtime_error, mutation_succeeded = example_lazy_mutation_guard()
    assert isinstance(first_extent, list)
    assert got_runtime_error is True
    assert mutation_succeeded is True
