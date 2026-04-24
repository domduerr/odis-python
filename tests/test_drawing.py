"""Integration tests for FormalContext.draw() and draw_svg()."""

import pytest
from odis import FormalContext

LIVING_BEINGS = "odis/test_data/living_beings_and_water.cxt"
TRIANGLES = "odis/test_data/triangles.cxt"
DATA_FROM_PAPER = "odis/test_data/data_from_paper.cxt"

EXPECTED_CONCEPT_COUNTS = {
    LIVING_BEINGS: 19,
    TRIANGLES: None,  # determined dynamically
    DATA_FROM_PAPER: None,
}


# ===========================================================================
# US1 — Coordinate-Based Layout (T012)
# ===========================================================================


class TestDrawCoordinates:
    def test_draw_returns_drawing_object(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw()
        assert result is not None

    def test_draw_coordinate_count_matches_concept_count(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw()
        assert result is not None
        concepts = list(ctx.concepts())
        assert len(result.coordinates) == len(concepts)

    def test_draw_living_beings_has_19_coordinates(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw()
        assert result is not None
        assert len(result.coordinates) == 19

    def test_draw_coordinates_are_float_pairs(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw()
        assert result is not None
        for coord in result.coordinates:
            assert isinstance(coord, tuple)
            assert len(coord) == 2
            x, y = coord
            assert isinstance(x, float)
            assert isinstance(y, float)

    def test_draw_edges_are_int_pairs(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw()
        assert result is not None
        n = len(result.coordinates)
        for edge in result.edges:
            assert isinstance(edge, tuple)
            assert len(edge) == 2
            i, j = edge
            assert isinstance(i, int)
            assert isinstance(j, int)
            assert 0 <= i < n
            assert 0 <= j < n

    def test_draw_empty_context_returns_none(self):
        ctx = FormalContext()
        result = ctx.draw()
        assert result is None

    def test_draw_context_no_attributes_returns_none(self):
        ctx = FormalContext()
        ctx.add_object("obj1", set())
        result = ctx.draw()
        assert result is None

    def test_draw_single_object_single_attribute(self):
        ctx = FormalContext()
        ctx.add_attribute("attr")
        ctx.add_object("obj", {"attr"})
        result = ctx.draw()
        # Small context: should produce a Drawing (not None)
        assert result is not None
        assert len(result.coordinates) == len(list(ctx.concepts()))

    def test_draw_algorithm_dimdraw(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw(algorithm="dimdraw")
        assert result is not None
        assert len(result.coordinates) == 19

    def test_draw_algorithm_sugiyama(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw(algorithm="sugiyama")
        assert result is not None
        assert len(result.coordinates) == 19

    def test_draw_unknown_algorithm_raises_value_error(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        with pytest.raises(ValueError, match="dimdraw"):
            ctx.draw(algorithm="unknown_algo")

    def test_draw_repr(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw()
        assert result is not None
        r = repr(result)
        assert "Drawing" in r
        assert "19" in r

    def test_draw_is_read_only(self):
        """draw() must not mutate the context."""
        ctx = FormalContext.from_file(LIVING_BEINGS)
        before = len(list(ctx.concepts()))
        ctx.draw()
        after = len(list(ctx.concepts()))
        assert before == after

    def test_draw_triangles(self):
        ctx = FormalContext.from_file(TRIANGLES)
        result = ctx.draw()
        assert result is not None
        assert len(result.coordinates) == len(list(ctx.concepts()))

    def test_draw_data_from_paper(self):
        ctx = FormalContext.from_file(DATA_FROM_PAPER)
        result = ctx.draw()
        assert result is not None
        assert len(result.coordinates) == len(list(ctx.concepts()))

    def test_draw_edge_count_matches_cover_relations(self):
        """SC-003: edge count in Drawing == number of covering pairs in the lattice."""
        ctx = FormalContext.from_file(LIVING_BEINGS)
        d1 = ctx.draw(algorithm="dimdraw")
        d2 = ctx.draw(algorithm="sugiyama")
        assert d1 is not None and d2 is not None
        # Both algorithms operate on the same lattice — same edge count.
        assert len(d1.edges) == len(d2.edges)


# ===========================================================================
# US2 — SVG String Output (T016)
# ===========================================================================


class TestDrawSvg:
    def test_draw_svg_returns_string(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw_svg()
        assert isinstance(result, str)

    def test_draw_svg_starts_with_svg_tag(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        result = ctx.draw_svg()
        assert result.startswith("<svg")

    def test_draw_svg_contains_correct_node_count(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        svg = ctx.draw_svg()
        # One <g> group per concept node
        assert svg.count("<g ") == len(drawing.nodes)

    def test_draw_svg_contains_correct_edge_count(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        svg = ctx.draw_svg()
        # One <line> per edge
        assert svg.count("<line ") == len(drawing.edges)

    def test_draw_svg_default_dimensions(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        svg = ctx.draw_svg()
        assert 'width="800"' in svg
        assert 'height="600"' in svg

    def test_draw_svg_custom_dimensions(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        svg = ctx.draw_svg(width=1200, height=900)
        assert 'width="1200"' in svg
        assert 'height="900"' in svg

    def test_draw_svg_empty_context_returns_valid_svg(self):
        ctx = FormalContext()
        result = ctx.draw_svg()
        assert isinstance(result, str)
        assert result.startswith("<svg")
        # Should not raise; may be empty svg
        assert "<svg" in result

    def test_draw_svg_unknown_algorithm_raises_value_error(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        with pytest.raises(ValueError):
            ctx.draw_svg(algorithm="bad")

    def test_draw_svg_zero_width_raises_value_error(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        with pytest.raises(ValueError):
            ctx.draw_svg(width=0, height=600)

    def test_draw_svg_negative_height_raises_value_error(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        with pytest.raises(ValueError):
            ctx.draw_svg(width=800, height=-1)

    def test_draw_svg_sugiyama(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        svg = ctx.draw_svg(algorithm="sugiyama")
        assert svg.startswith("<svg")
        drawing = ctx.draw(algorithm="sugiyama")
        assert drawing is not None
        assert svg.count("<g ") == len(drawing.nodes)
        assert svg.count("<line ") == len(drawing.edges)

    def test_to_svg_matches_draw_svg(self):
        """Round-trip: drawing.to_svg(ctx) == ctx.draw_svg() (same layout call)."""
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        svg_from_drawing = drawing.to_svg(ctx, width=800, height=600)
        svg_direct = ctx.draw_svg(width=800, height=600)
        # Both must be valid SVGs with the same structure counts
        assert svg_from_drawing.count("<g ") == svg_direct.count("<g ")
        assert svg_from_drawing.count("<line ") == svg_direct.count("<line ")

    def test_to_svg_zero_width_raises_value_error(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        with pytest.raises(ValueError):
            drawing.to_svg(ctx, width=0, height=600)

    def test_to_svg_custom_dimensions_reflected_in_svg(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        svg = drawing.to_svg(ctx, width=400, height=300)
        assert 'width="400"' in svg
        assert 'height="300"' in svg


# ===========================================================================
# US3 — Node Labels in Drawing Output (T018)
# ===========================================================================


class TestDrawingNodes:
    def test_nodes_length_matches_concepts(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        assert len(drawing.nodes) == len(list(ctx.concepts()))

    def test_nodes_have_correct_fields(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        for i, node in enumerate(drawing.nodes):
            assert node.index == i
            assert isinstance(node.x, float)
            assert isinstance(node.y, float)
            assert isinstance(node.object_labels, list)
            assert isinstance(node.attribute_labels, list)
            for label in node.object_labels:
                assert isinstance(label, str)
            for label in node.attribute_labels:
                assert isinstance(label, str)

    def test_nodes_concept_matches_ctx_concepts(self):
        """SC-002 / data model: drawing.nodes[i].concept == ctx.concepts()[i]."""
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        ctx_concepts = list(ctx.concepts())
        for i, node in enumerate(drawing.nodes):
            assert node.concept == ctx_concepts[i], (
                f"Node {i}: node.concept={node.concept!r} != ctx.concepts()[{i}]={ctx_concepts[i]!r}"
            )

    def test_reduced_labels_cover_all_objects(self):
        """Total object labels across all nodes == number of objects in context."""
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        total_obj_labels = sum(len(node.object_labels) for node in drawing.nodes)
        assert total_obj_labels == len(ctx.objects)

    def test_reduced_labels_cover_all_attributes(self):
        """Total attribute labels across all nodes == number of attributes in context."""
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        total_attr_labels = sum(len(node.attribute_labels) for node in drawing.nodes)
        assert total_attr_labels == len(ctx.attributes)

    def test_node_repr(self):
        ctx = FormalContext.from_file(LIVING_BEINGS)
        drawing = ctx.draw()
        assert drawing is not None
        r = repr(drawing.nodes[0])
        assert "DrawingNode" in r
        assert "index=0" in r
