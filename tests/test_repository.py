"""Tests for loading contexts from the FCA repository.

These hit https://fcarepository.org/ over the network and are skipped when it is
not reachable, so that an offline run of the suite still passes.
"""
import pytest
import odis


@pytest.fixture(scope="module")
def catalog() -> list[odis.RepositoryEntry]:
    try:
        return odis.repository_catalog()
    except ConnectionError as err:
        pytest.skip(f"FCA repository not reachable: {err}")


def test_catalog_is_not_empty(catalog):
    assert len(catalog) > 0
    assert all(e.filename.endswith(".cxt") for e in catalog)
    assert all(e.title for e in catalog)


def test_catalog_entry_fields(catalog):
    entry = next(e for e in catalog if e.filename == "livingbeings_en.cxt")
    assert entry.title == "Living beings and water"
    assert entry.objects == 8
    assert entry.attributes == 9
    assert entry.language == "English"
    assert entry.source
    assert entry.url.endswith("/contexts/livingbeings_en.cxt")


def test_from_repository(catalog):
    ctx = odis.FormalContext.from_repository("livingbeings_en.cxt")
    assert ctx.shape == (8, 9)
    assert "frog" in ctx
    assert ctx["frog", "lives on land"]


def test_entry_load_matches_catalog_metadata(catalog):
    entry = next(e for e in catalog if e.filename == "livingbeings_en.cxt")
    ctx = entry.load()
    assert ctx.shape == (entry.objects, entry.attributes)


def test_loaded_context_is_usable(catalog):
    ctx = odis.FormalContext.from_repository("livingbeings_en.cxt")
    assert len(ctx.concepts()) == 19


def test_unknown_context_raises(catalog):
    with pytest.raises(ConnectionError):
        odis.FormalContext.from_repository("no_such_context.cxt")
