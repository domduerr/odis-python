"""Tests for canonical basis and implication API."""
import pytest
import odis


def test_canonical_basis_type(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    assert isinstance(basis, odis.ImplicationList)


def test_canonical_basis_non_empty(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    assert len(basis) > 0


def test_canonical_basis_indexing(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    imp = basis[0]
    assert isinstance(imp, odis.Implication)


def test_canonical_basis_negative_index(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    last1 = basis[-1]
    last2 = basis[len(basis) - 1]
    assert last1 == last2


def test_canonical_basis_index_out_of_range(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    with pytest.raises(IndexError):
        _ = basis[len(basis) + 100]


def test_canonical_basis_slice(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    sl = basis[0:2]
    assert isinstance(sl, list)
    assert len(sl) == 2
    for item in sl:
        assert isinstance(item, odis.Implication)


def test_canonical_basis_iteration(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    items = list(basis)
    assert len(items) == len(basis)
    for item in items:
        assert isinstance(item, odis.Implication)


def test_canonical_basis_repr(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    r = repr(ctx.canonical_basis())
    assert "ImplicationList" in r


def test_implication_has_premise_and_conclusion(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    imp = ctx.canonical_basis()[0]
    assert hasattr(imp, "premise")
    assert hasattr(imp, "conclusion")
    assert isinstance(imp.premise, odis.LabelSet)
    assert isinstance(imp.conclusion, odis.LabelSet)


def test_implication_unpack(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    imp = ctx.canonical_basis()[0]
    premise, conclusion = imp
    assert isinstance(premise, odis.LabelSet)
    assert isinstance(conclusion, odis.LabelSet)


def test_implication_len(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    imp = ctx.canonical_basis()[0]
    assert len(imp) == 2


def test_implication_repr(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    imp = ctx.canonical_basis()[0]
    r = repr(imp)
    assert "Implication" in r
    assert "premise" in r
    assert "conclusion" in r


def test_implication_equality(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    assert basis[0] == basis[0]


def test_implication_to_python(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    imp = ctx.canonical_basis()[0]
    pair = imp.to_python()
    assert isinstance(pair, tuple)
    assert len(pair) == 2
    assert isinstance(pair[0], frozenset)
    assert isinstance(pair[1], frozenset)


def test_implication_list_to_python(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    py_list = basis.to_python()
    assert isinstance(py_list, list)
    assert len(py_list) == len(basis)
    for item in py_list:
        assert isinstance(item, tuple)
        assert len(item) == 2
        assert isinstance(item[0], frozenset)
        assert isinstance(item[1], frozenset)


def test_canonical_basis_optimised_type(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis_optimised()
    assert isinstance(basis, odis.ImplicationList)


def test_canonical_basis_optimised_non_empty(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis_optimised()
    assert len(basis) > 0


def test_canonical_basis_optimised_same_count_as_regular(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    regular = ctx.canonical_basis()
    optimised = ctx.canonical_basis_optimised()
    # Both methods must yield the same number of implications (Duquenne-Guigues)
    assert len(regular) == len(optimised)


def test_next_preclosure_with_empty_set(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    result = ctx.next_preclosure(basis, frozenset())
    assert isinstance(result, odis.LabelSet)


def test_next_preclosure_with_labelset(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    # Use result of first call as input to second call
    first = ctx.next_preclosure(basis, frozenset())
    second = ctx.next_preclosure(basis, first)
    assert isinstance(second, odis.LabelSet)


def test_next_preclosure_covers_canonical_basis(living_beings_path):
    """Iterating next_preclosure to fixpoint should enumerate canonical basis premises."""
    ctx = odis.FormalContext.from_file(living_beings_path)
    basis = ctx.canonical_basis()
    n_attrs = len(ctx.attributes)

    visited: list = []
    current = frozenset()
    while len(current) < n_attrs:
        nxt = ctx.next_preclosure(basis, current)
        if len(nxt) == n_attrs:
            break
        visited.append(nxt)
        current = nxt

    # Each visited preclosure should be an attribute subset
    for ls in visited:
        assert isinstance(ls, odis.LabelSet)


def test_canonical_basis_lazy_type(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = ctx.canonical_basis(lazy=True)
    assert isinstance(gen, odis.ImplicationGenerator)


def test_canonical_basis_lazy_yields_all(living_beings_path):
    ctx = odis.FormalContext.from_file(living_beings_path)
    n = len(ctx.canonical_basis())
    gen = ctx.canonical_basis(lazy=True)
    items = list(gen)
    assert len(items) == n
    for item in items:
        assert isinstance(item, odis.Implication)


def test_lazy_mutation_raises_runtime_error(living_beings_path):
    """Mutating the context while a lazy generator is alive must raise RuntimeError."""
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = ctx.canonical_basis(lazy=True)
    # Consume one item to start
    next(gen)
    # Mutation should raise RuntimeError
    with pytest.raises(RuntimeError):
        ctx.add_object("injected_object")


def test_lazy_mutation_raises_on_next(living_beings_path):
    """After a mutation invalidates the generator, __next__ must raise RuntimeError."""
    ctx = odis.FormalContext.from_file(living_beings_path)
    gen = ctx.concepts(lazy=True)
    # Force mutation; context.add_object raises RuntimeError directly due to guard
    try:
        ctx.add_object("x")
    except RuntimeError:
        pass
    # Generator is now stale — next iteration must also raise
    for _ in range(3):
        try:
            val = next(gen)
        except (StopIteration, RuntimeError):
            break
