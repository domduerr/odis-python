"""Shared pytest fixtures for odis-python tests."""
from pathlib import Path
import pytest

# Absolute path to the workspace root's test data directory.
_TEST_DATA_DIR = Path(__file__).parent.parent.parent / "odis" / "test_data"


@pytest.fixture
def living_beings_path() -> str:
    """Absolute path to living_beings_and_water.cxt."""
    p = _TEST_DATA_DIR / "living_beings_and_water.cxt"
    assert p.exists(), f"Test data file not found: {p}"
    return str(p)


@pytest.fixture
def data_from_paper_path() -> str:
    """Absolute path to data_from_paper.cxt."""
    p = _TEST_DATA_DIR / "data_from_paper.cxt"
    assert p.exists(), f"Test data file not found: {p}"
    return str(p)


@pytest.fixture
def triangles_path() -> str:
    """Absolute path to triangles.cxt."""
    p = _TEST_DATA_DIR / "triangles.cxt"
    assert p.exists(), f"Test data file not found: {p}"
    return str(p)
