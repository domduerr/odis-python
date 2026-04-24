"""Tests for Titanic iceberg concept lattice."""
import pytest
import odis


def test_titanic_constructor():
    t = odis.Titanic()
    assert t is not None


def test_enumerate_returns_collection(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = odis.Titanic().enumerate(ctx, min_support=3)
    assert isinstance(coll, odis.ConceptCollection)


def test_enumerate_all_extents_meet_support(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    min_support = 3
    coll = odis.Titanic().enumerate(ctx, min_support=min_support)
    for c in coll:
        assert len(c.extent) >= min_support, (
            f"Concept extent has {len(c.extent)} objects, expected >= {min_support}: "
            f"{list(c.extent)}"
        )


def test_enumerate_min_support_above_object_count_returns_empty(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    n_objects = len(ctx.objects)
    coll = odis.Titanic().enumerate(ctx, min_support=n_objects + 1)
    assert isinstance(coll, odis.ConceptCollection)
    assert len(coll) == 0


def test_enumerate_min_support_zero_equals_all_concepts(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    all_concepts = ctx.concepts()
    iceberg = odis.Titanic().enumerate(ctx, min_support=0)
    assert len(iceberg) == len(all_concepts)


def test_enumerate_lazy_returns_generator(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = odis.Titanic().enumerate(ctx, min_support=3, lazy=True)
    assert isinstance(gen, odis.ConceptGenerator)


def test_enumerate_lazy_yields_correct_count(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    min_support = 3
    expected = len(odis.Titanic().enumerate(ctx, min_support=min_support))
    gen = odis.Titanic().enumerate(ctx, min_support=min_support, lazy=True)
    items = list(gen)
    assert len(items) == expected
    for item in items:
        assert isinstance(item, odis.Concept)
