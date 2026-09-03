"""Tests for FormalContext.from_file and concept enumeration."""
import pytest
import odis

def test_from_file_loads_context(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert ctx is not None


def test_from_file_objects(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    objects = ctx.objects
    assert isinstance(objects, list)
    assert len(objects) == 8
    assert "fish leech" in objects


def test_from_file_attributes(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    attrs = ctx.attributes
    assert isinstance(attrs, list)
    assert len(attrs) == 9
    assert "needs water to live" in attrs


def test_from_file_not_found_raises():
    with pytest.raises(FileNotFoundError):
        odis.FormalContext.from_file("/does/not/exist.cxt")


def test_from_file_invalid_content(tmp_path):
    bad = tmp_path / "bad.cxt"
    bad.write_bytes(b"NOT A VALID CXT FILE\x00\x01\x02")
    with pytest.raises(ValueError):
        odis.FormalContext.from_file(str(bad))




def test_shape(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert ctx.shape == (8, 9)


def test_len(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert len(ctx) == 8


def test_contains_object(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert "frog" in ctx
    assert "unicorn" not in ctx


def test_repr(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    r = repr(ctx)
    assert "FormalContext" in r




def test_getitem_true(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    # "frog" has attribute "needs water to live"
    assert ctx["frog", "needs water to live"] is True


def test_getitem_false(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    # "dog" does not live in water
    assert ctx["dog", "lives in water"] is False



def test_concepts_returns_collection(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    assert isinstance(coll, odis.ConceptCollection)


def test_concepts_count(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    assert len(coll) == 19, f"Expected 19 formal concepts, got {len(coll)}"


def test_concepts_collection_indexing(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    c = coll[0]
    assert isinstance(c, odis.Concept)


def test_concepts_collection_negative_index(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    last1 = coll[-1]
    last2 = coll[len(coll) - 1]
    assert last1 == last2


def test_concepts_collection_index_out_of_range(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    with pytest.raises(IndexError):
        _ = coll[100]


def test_concepts_collection_slice(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    sl = coll[0:3]
    assert isinstance(sl, list)
    assert len(sl) == 3
    for item in sl:
        assert isinstance(item, odis.Concept)


def test_concepts_collection_iteration(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    items = list(coll)
    assert len(items) == 19
    for item in items:
        assert isinstance(item, odis.Concept)


def test_concept_has_extent_and_intent(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    c = coll[0]
    assert hasattr(c, "extent")
    assert hasattr(c, "intent")
    assert isinstance(c.extent, odis.LabelSet)
    assert isinstance(c.intent, odis.LabelSet)


def test_concept_unpack(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    c = coll[0]
    extent, intent = c
    assert isinstance(extent, odis.LabelSet)
    assert isinstance(intent, odis.LabelSet)


def test_concept_len(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    c = ctx.concepts()[0]
    assert len(c) == 2


def test_concept_equality(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    assert coll[0] == coll[0]
    # Different concepts should not be equal (unless the context has duplicates)
    if len(coll) > 1:
        # At least one pair must differ
        found_diff = any(coll[i] != coll[j] for i in range(len(coll)) for j in range(i + 1, len(coll)))
        assert found_diff


def test_concept_repr(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    c = ctx.concepts()[0]
    r = repr(c)
    assert "Concept" in r
    assert "extent" in r
    assert "intent" in r


def test_concept_contains(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    # Find a concept that has non-empty extent
    found = False
    for c in coll:
        labels = list(c.extent)
        if labels:
            assert labels[0] in c
            found = True
            break
    assert found, "All concepts have empty extent"


def test_labelset_iter(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    # The bottom concept (empty intent) should have full extent or near full
    for c in coll:
        for label in c.extent:
            assert isinstance(label, str)
        break


def test_labelset_len(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    for c in coll:
        assert isinstance(len(c.extent), int)
        assert isinstance(len(c.intent), int)


def test_labelset_contains(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    for c in coll:
        for label in c.extent:
            assert label in c.extent
            break
        break


def test_labelset_to_frozenset(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    coll = ctx.concepts()
    c = coll[0]
    fs = c.extent.to_frozenset()
    assert isinstance(fs, frozenset)



def test_concept_to_python(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    c = ctx.concepts()[0]
    pair = c.to_python()
    assert isinstance(pair, tuple)
    assert len(pair) == 2
    assert isinstance(pair[0], frozenset)
    assert isinstance(pair[1], frozenset)


def test_collection_to_python(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    py_list = ctx.concepts().to_python()
    assert isinstance(py_list, list)
    assert len(py_list) == 19
    for item in py_list:
        assert isinstance(item, tuple)
        assert len(item) == 2
        assert isinstance(item[0], frozenset)
        assert isinstance(item[1], frozenset)


def test_concepts_sorted(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    sorted_coll = ctx.concepts().sorted()
    assert isinstance(sorted_coll, odis.ConceptCollection)
    assert len(sorted_coll) == 19


def test_collection_repr(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    r = repr(ctx.concepts())
    assert "ConceptCollection" in r
    assert "19" in r


def test_concepts_lazy_returns_generator(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = ctx.concepts(lazy=True)
    assert isinstance(gen, odis.ConceptGenerator)


def test_concepts_lazy_yields_all(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = ctx.concepts(lazy=True)
    items = list(gen)
    assert len(items) == 19
    for item in items:
        assert isinstance(item, odis.Concept)
