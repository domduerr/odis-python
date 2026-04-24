"""Tests for FormalContext programmatic build API."""
import pytest
import odis



def test_empty_constructor():
    ctx = odis.FormalContext()
    assert ctx.shape == (0, 0)
    assert ctx.objects == []
    assert ctx.attributes == []




def test_from_dict_basic():
    ctx = odis.FormalContext.from_dict({"cat": {"legs", "fur"}, "fish": {"scales"}})
    assert "cat" in ctx
    assert "fish" in ctx
    assert "legs" in ctx.attributes
    assert "fur" in ctx.attributes
    assert "scales" in ctx.attributes


def test_from_dict_shape():
    ctx = odis.FormalContext.from_dict({"a": {"x", "y"}, "b": {"y", "z"}})
    n, m = ctx.shape
    assert n == 2
    assert m == 3


def test_from_dict_incidence():
    ctx = odis.FormalContext.from_dict({"bird": {"flies", "has_wings"}})
    assert ctx["bird", "flies"] is True
    assert ctx["bird", "has_wings"] is True


def test_from_dict_empty():
    ctx = odis.FormalContext.from_dict({})
    assert ctx.shape == (0, 0)


def test_add_object_appears_in_objects():
    ctx = odis.FormalContext()
    ctx.add_object("bird")
    assert "bird" in ctx.objects


def test_add_object_with_attributes():
    ctx = odis.FormalContext()
    ctx.add_attribute("flies")
    ctx.add_attribute("swims")
    ctx.add_object("duck", {"flies", "swims"})
    assert ctx["duck", "flies"] is True
    assert ctx["duck", "swims"] is True


def test_add_object_auto_creates_attributes():
    ctx = odis.FormalContext()
    ctx.add_object("hawk", {"flies"})
    assert "flies" in ctx.attributes
    assert ctx["hawk", "flies"] is True


def test_add_object_duplicate_raises():
    ctx = odis.FormalContext()
    ctx.add_object("hawk")
    with pytest.raises(ValueError):
        ctx.add_object("hawk")


def test_add_attribute_appears_in_attributes():
    ctx = odis.FormalContext()
    ctx.add_attribute("warm_blooded")
    assert "warm_blooded" in ctx.attributes


def test_add_attribute_duplicate_raises():
    ctx = odis.FormalContext()
    ctx.add_attribute("x")
    with pytest.raises(ValueError):
        ctx.add_attribute("x")


def test_remove_object():
    ctx = odis.FormalContext.from_dict({"a": {"x"}, "b": {}})
    ctx.remove_object("a")
    assert "a" not in ctx.objects
    assert "b" in ctx.objects


def test_remove_object_not_found_raises():
    ctx = odis.FormalContext()
    with pytest.raises(KeyError):
        ctx.remove_object("nonexistent")


def test_remove_attribute():
    ctx = odis.FormalContext.from_dict({"a": {"x", "y"}})
    ctx.remove_attribute("x")
    assert "x" not in ctx.attributes
    assert "y" in ctx.attributes


def test_remove_attribute_not_found_raises():
    ctx = odis.FormalContext()
    with pytest.raises(KeyError):
        ctx.remove_attribute("nonexistent")


def test_rename_object():
    ctx = odis.FormalContext.from_dict({"old_name": {"x"}})
    ctx.rename_object("old_name", "new_name")
    assert "new_name" in ctx.objects
    assert "old_name" not in ctx.objects


def test_rename_object_not_found_raises():
    ctx = odis.FormalContext()
    with pytest.raises(KeyError):
        ctx.rename_object("ghost", "new_name")


def test_rename_attribute():
    ctx = odis.FormalContext.from_dict({"a": {"old_attr"}})
    ctx.rename_attribute("old_attr", "new_attr")
    assert "new_attr" in ctx.attributes
    assert "old_attr" not in ctx.attributes


def test_getitem():
    ctx = odis.FormalContext.from_dict({"bird": {"flies"}})
    assert ctx["bird", "flies"] is True
    assert ctx["bird", "flies"] is not False


def test_setitem_add_incidence():
    ctx = odis.FormalContext.from_dict({"hawk": {"flies"}, "duck": {}})
    # duck does not fly initially
    assert ctx["duck", "flies"] is False
    ctx["duck", "flies"] = True
    assert ctx["duck", "flies"] is True


def test_setitem_remove_incidence():
    ctx = odis.FormalContext.from_dict({"hawk": {"flies"}})
    assert ctx["hawk", "flies"] is True
    ctx["hawk", "flies"] = False
    assert ctx["hawk", "flies"] is False


def test_getitem_unknown_object_raises():
    ctx = odis.FormalContext.from_dict({"a": {"x"}})
    with pytest.raises(KeyError):
        _ = ctx["unknown", "x"]


def test_getitem_unknown_attribute_raises():
    ctx = odis.FormalContext.from_dict({"a": {"x"}})
    with pytest.raises(KeyError):
        _ = ctx["a", "unknown"]


def test_copy_is_independent():
    ctx = odis.FormalContext.from_dict({"a": {"x"}})
    cp = ctx.copy()
    cp.add_object("b")
    # Original unchanged
    assert "b" not in ctx.objects
    assert "b" in cp.objects


def test_copy_has_same_data():
    ctx = odis.FormalContext.from_dict({"a": {"x"}, "b": {"y"}})
    cp = ctx.copy()
    assert cp.objects == ctx.objects
    assert cp.attributes == ctx.attributes
    assert cp["a", "x"] == ctx["a", "x"]


def test_to_file_round_trip(tmp_path, living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    out = tmp_path / "out.cxt"
    ctx.to_file(str(out))
    ctx2 = odis.FormalContext.from_file(str(out))
    assert sorted(ctx.objects) == sorted(ctx2.objects)
    assert sorted(ctx.attributes) == sorted(ctx2.attributes)


def test_extent_empty_set_returns_all_objects(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    ext = ctx.extent(frozenset())
    # Empty attribute set → all objects are in the extent
    assert len(ext) == len(ctx.objects)


def test_intent_empty_set_returns_all_attributes(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    intent = ctx.intent(frozenset())
    # Empty object set → intent is all attributes (vacuous satisfaction)
    assert len(intent) == len(ctx.attributes)


def test_extent_single_attribute(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    # "breast feeds" — only dog in this context
    ext = ctx.extent({"breast feeds"})
    assert "dog" in ext
    assert len(ext) >= 1


def test_intent_returns_labelset(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    intent = ctx.intent({"frog"})
    assert isinstance(intent, odis.LabelSet)


def test_attribute_hull(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    hull = ctx.attribute_hull(frozenset())
    assert isinstance(hull, odis.LabelSet)


def test_object_hull(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    hull = ctx.object_hull(frozenset())
    assert isinstance(hull, odis.LabelSet)
