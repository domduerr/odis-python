"""Type-level tests for the documented call shapes.

Checked statically by `mypy --strict` in CI, and executed under pytest as a
smoke test that the shapes the stubs promise are the shapes that actually work.
`assert_type` is a no-op at runtime, so this file serves both purposes.
"""

from typing_extensions import assert_type

import odis


def test_context_shapes(living_beings_path: str) -> None:
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert_type(ctx, odis.FormalContext)
    assert_type(ctx.shape, tuple[int, int])
    assert_type(ctx.objects, list[str])
    assert_type(ctx.attributes, list[str])
    assert_type(ctx.name, str)
    assert_type(len(ctx), int)
    assert_type(ctx["frog", "needs water to live"], bool)


def test_concept_shapes(living_beings_path: str) -> None:
    ctx = odis.FormalContext.from_file(living_beings_path)

    # `lazy` selects the return type via overload
    assert_type(ctx.concepts(), odis.ConceptCollection)
    assert_type(ctx.concepts(lazy=True), odis.ConceptGenerator)

    concepts = ctx.concepts()
    assert_type(concepts[0], odis.Concept)
    assert_type(concepts[0:2], list[odis.Concept])
    assert_type(concepts[-1], odis.Concept)

    extent, intent = concepts[0]
    assert_type(extent, odis.LabelSet)
    assert_type(intent, odis.LabelSet)


def test_implication_shapes(living_beings_path: str) -> None:
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert_type(ctx.canonical_basis(), odis.ImplicationList)
    assert_type(ctx.canonical_basis(lazy=True), odis.ImplicationGenerator)
    assert_type(ctx.canonical_basis_optimised(), odis.ImplicationList)

    basis = ctx.canonical_basis()
    assert_type(basis[0], odis.Implication)
    assert_type(basis[0].premise, odis.LabelSet)


def test_derivation_shapes(living_beings_path: str) -> None:
    ctx = odis.FormalContext.from_file(living_beings_path)
    # a plain iterable of names is accepted, as is a LabelSet
    assert_type(ctx.intent({"frog"}), odis.LabelSet)
    assert_type(ctx.extent(["needs water to live"]), odis.LabelSet)
    got = ctx.intent({"frog"})
    assert_type(ctx.extent(got), odis.LabelSet)
    assert_type(got.to_frozenset(), frozenset[str])


def test_drawing_shapes(living_beings_path: str) -> None:
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert_type(ctx.draw(), odis.Drawing | None)
    assert_type(ctx.draw_svg(), str)

    drawing = ctx.draw()
    assert drawing is not None
    assert_type(drawing.coordinates, list[tuple[float, float]])
    assert_type(drawing.nodes[0].x, float)
    assert_type(drawing.nodes[0].object_labels, list[str])

    poset = odis.Poset(["a", "b"], [(0, 1)])
    assert_type(poset.draw_svg(), str)
    assert_type(poset.nodes, list[str])


def test_titanic_shapes(living_beings_path: str) -> None:
    ctx = odis.FormalContext.from_file(living_beings_path)
    assert_type(odis.Titanic().enumerate(ctx, 1), odis.ConceptCollection)
    assert_type(
        odis.Titanic().enumerate(ctx, 1, lazy=True), odis.ConceptGenerator
    )
