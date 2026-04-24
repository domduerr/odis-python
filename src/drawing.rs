use std::fmt::Write;
use std::sync::Arc;

use bit_set::BitSet;
use pyo3::prelude::*;

use crate::concept::Concept;
use crate::context::PyFormalContext;
use crate::labelset::LabelSet;

// ---------------------------------------------------------------------------
// DrawingNode — one positioned concept node.
// Fields are exposed to Python in US3 (T017); stored as pub(crate) for now.
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Clone)]
pub struct DrawingNode {
    #[pyo3(get)]
    pub(crate) index: usize,
    #[pyo3(get)]
    pub(crate) x: f64,
    #[pyo3(get)]
    pub(crate) y: f64,
    #[pyo3(get)]
    pub(crate) concept: Concept,
    #[pyo3(get)]
    pub(crate) object_labels: Vec<String>,
    #[pyo3(get)]
    pub(crate) attribute_labels: Vec<String>,
}

#[pymethods]
impl DrawingNode {
    fn __repr__(&self) -> String {
        format!(
            "DrawingNode(index={}, x={:.1}, y={:.1})",
            self.index, self.x, self.y
        )
    }
}

// ---------------------------------------------------------------------------
// Drawing — layout result returned by FormalContext.draw().
// `coordinates` and `edges` are exposed to Python; `nodes` in US3 (T017).
// ---------------------------------------------------------------------------

#[pyclass]
pub struct Drawing {
    #[pyo3(get)]
    pub(crate) coordinates: Vec<(f64, f64)>,
    #[pyo3(get)]
    pub(crate) edges: Vec<(u32, u32)>,
    #[pyo3(get)]
    pub(crate) nodes: Vec<DrawingNode>,
}

#[pymethods]
impl Drawing {
    fn __repr__(&self) -> String {
        format!(
            "Drawing(nodes={}, edges={})",
            self.nodes.len(),
            self.edges.len()
        )
    }

    /// Render this drawing as an SVG string scaled to the given viewport.
    ///
    /// `ctx` is accepted for API symmetry with `FormalContext.draw_svg()`;
    /// the reduced labels are already stored on each node and do not require
    /// a second context lookup.
    #[pyo3(signature = (ctx, width=800, height=600))]
    fn to_svg(&self, ctx: &PyFormalContext, width: i64, height: i64) -> PyResult<String> {
        let _ = ctx;
        if width <= 0 || height <= 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "width and height must be positive integers",
            ));
        }
        Ok(render_svg(self, width as usize, height as usize))
    }
}

// ---------------------------------------------------------------------------
// SVG rendering (private)
// ---------------------------------------------------------------------------

/// Public wrapper around `render_svg` for use by `context.rs::draw_svg()`.
pub fn render_svg_pub(drawing: &Drawing, width: usize, height: usize) -> String {
    render_svg(drawing, width, height)
}

fn render_svg(drawing: &Drawing, width: usize, height: usize) -> String {
    let mut svg = String::new();
    let w = width;
    let h = height;

    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">"
    );

    if drawing.nodes.is_empty() {
        svg.push_str("</svg>");
        return svg;
    }

    let scaled = odis::Drawing::new(drawing.coordinates.clone())
        .scale_to_viewport(width as f64, height as f64, 40.0);

    // Edges (lines) first so nodes render on top.
    for &(from, to) in &drawing.edges {
        if let (Some(&(x1, y1)), Some(&(x2, y2))) =
            (scaled.get(from as usize), scaled.get(to as usize))
        {
            let _ = write!(
                svg,
                "<line x1=\"{x1:.1}\" y1=\"{y1:.1}\" x2=\"{x2:.1}\" y2=\"{y2:.1}\" stroke=\"#666666\" stroke-width=\"1.5\"/>"
            );
        }
    }

    // Nodes (groups with circle + reduced text labels).
    for node in &drawing.nodes {
        if let Some(&(x, y)) = scaled.get(node.index) {
            let obj_text = html_escape(&node.object_labels.join(", "));
            let attr_text = html_escape(&node.attribute_labels.join(", "));
            let _ = write!(
                svg,
                "<g transform=\"translate({x:.1},{y:.1})\"><circle r=\"8\" fill=\"#ffffff\" stroke=\"#333333\" stroke-width=\"2\"/><text dy=\"-12\" text-anchor=\"middle\" font-size=\"12\" font-family=\"sans-serif\">{obj_text}</text><text dy=\"20\" text-anchor=\"middle\" font-size=\"12\" font-family=\"sans-serif\">{attr_text}</text></g>"
            );
        }
    }

    svg.push_str("</svg>");
    svg
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// make_drawing — shared builder called by context.rs draw() and draw_svg()
// ---------------------------------------------------------------------------

pub fn make_drawing(ctx: &PyFormalContext, algorithm: &str) -> PyResult<Option<Drawing>> {
    use odis::algorithms::{dimdraw::DimDraw, sugiyama::Sugiyama};
    use odis::DrawingAlgorithm;

    // Validate algorithm before doing any FCA work.
    if algorithm != "dimdraw" && algorithm != "sugiyama" {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown algorithm '{}'. Valid values: 'dimdraw', 'sugiyama'",
            algorithm
        )));
    }

    // Degenerate context: nothing to draw.
    if ctx.inner.objects.is_empty() || ctx.inner.attributes.is_empty() {
        return Ok(None);
    }

    let lattice = match ctx.inner.concept_lattice() {
        Some(l) => l,
        None => return Ok(None),
    };

    let raw_drawing = match algorithm {
        "dimdraw" => DimDraw { timeout_ms: 0 }.draw(&lattice),
        "sugiyama" => Sugiyama { vertex_spacing: 1 }.draw(&lattice),
        other => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown algorithm '{}'. Valid values: 'dimdraw', 'sugiyama'",
                other
            )))
        }
    };

    let raw_drawing = match raw_drawing {
        Some(d) => d,
        None => return Ok(None),
    };

    let reduced = ctx.inner.reduced_labels(&lattice);

    // Build Concept objects aligned with lattice.poset.nodes order,
    // which matches ctx.concepts() iteration order.
    let concepts: Vec<Concept> = lattice
        .poset
        .nodes
        .iter()
        .map(|(extent_bits, intent_bits)| Concept {
            extent: LabelSet {
                bits: extent_bits.clone(),
                labels: Arc::clone(&ctx.arc_objects),
            },
            intent: LabelSet {
                bits: intent_bits.clone(),
                labels: Arc::clone(&ctx.arc_attributes),
            },
        })
        .collect();

    let coordinates = raw_drawing.coordinates.clone();
    let edges = lattice.poset.covering_edges.clone();

    let nodes: Vec<DrawingNode> = raw_drawing
        .coordinates
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            let (obj_labels, attr_labels) = reduced.get(i).cloned().unwrap_or_default();
            DrawingNode {
                index: i,
                x,
                y,
                concept: concepts[i].clone(),
                object_labels: obj_labels,
                attribute_labels: attr_labels,
            }
        })
        .collect();

    Ok(Some(Drawing {
        coordinates,
        edges,
        nodes,
    }))
}

// ---------------------------------------------------------------------------
// PyPoset — a Python-accessible partial order for direct drawing.
// ---------------------------------------------------------------------------

/// A partial order that can be drawn directly without a `FormalContext`.
///
/// Nodes are arbitrary string labels; edges are pairs of 0-based indices
/// representing the covering relation (diagram edges): `(u, v)` means
/// u is covered by v (u ≺ v).
///
/// # Examples
///
/// ```python
/// from odis import Poset
///
/// # Diamond lattice: bottom(0) ≺ left(1), bottom(0) ≺ right(2),
/// #                  left(1) ≺ top(3),   right(2) ≺ top(3)
/// p = Poset(["bottom", "left", "right", "top"], [(0,1),(0,2),(1,3),(2,3)])
/// svg = p.draw_svg("dimdraw", width=400, height=400)
/// ```
#[pyclass(name = "Poset")]
pub struct PyPoset {
    nodes: Vec<String>,
    edges: Vec<(u32, u32)>,
}

#[pymethods]
impl PyPoset {
    /// Create a new `Poset`.
    ///
    /// * `nodes`  — list of node labels (strings), indexed from 0.
    /// * `edges`  — list of `(u, v)` pairs representing the covering relation
    ///              (u ≺ v, i.e. u is directly below v in the diagram).
    ///              Both indices must be valid positions in `nodes`.
    ///              Cycles are rejected with `ValueError`.
    #[new]
    fn new(nodes: Vec<String>, edges: Vec<(u32, u32)>) -> Self {
        PyPoset { nodes, edges }
    }

    /// The node labels of this poset (read-only).
    #[getter]
    fn nodes(&self) -> Vec<String> {
        self.nodes.clone()
    }

    /// The covering edges of this poset as a list of `(u, v)` index pairs.
    #[getter]
    fn edges(&self) -> Vec<(u32, u32)> {
        self.edges.clone()
    }

    /// Compute a layout and return a `Drawing` object.
    ///
    /// `algorithm` is `"dimdraw"` (default) or `"sugiyama"`.
    /// Returns `None` only when the poset is empty.
    #[pyo3(signature = (algorithm = "dimdraw"))]
    fn draw(&self, py: Python<'_>, algorithm: &str) -> PyResult<Option<Py<Drawing>>> {
        match make_poset_drawing(self, algorithm)? {
            Some(d) => Ok(Some(Py::new(py, d)?)),
            None => Ok(None),
        }
    }

    /// Compute a layout and render it directly as an SVG string.
    ///
    /// `algorithm` is `"dimdraw"` (default) or `"sugiyama"`.
    /// Returns an empty SVG for an empty poset.
    #[pyo3(signature = (algorithm = "dimdraw", width = 800, height = 600))]
    fn draw_svg(&self, algorithm: &str, width: i64, height: i64) -> PyResult<String> {
        if width <= 0 || height <= 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "width and height must be positive integers",
            ));
        }
        match make_poset_drawing(self, algorithm)? {
            Some(d) => Ok(render_svg_pub(&d, width as usize, height as usize)),
            None => Ok(format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\"></svg>"
            )),
        }
    }

    fn __repr__(&self) -> String {
        format!("Poset(nodes={}, edges={})", self.nodes.len(), self.edges.len())
    }
}

// ---------------------------------------------------------------------------
// make_poset_drawing — builder for PyPoset.draw() / draw_svg()
// ---------------------------------------------------------------------------

fn make_poset_drawing(poset_py: &PyPoset, algorithm: &str) -> PyResult<Option<Drawing>> {
    use odis::algorithms::{dimdraw::DimDraw, sugiyama::Sugiyama};
    use odis::{DrawingAlgorithm, Poset};

    if algorithm != "dimdraw" && algorithm != "sugiyama" {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Unknown algorithm '{}'. Valid values: 'dimdraw', 'sugiyama'",
            algorithm
        )));
    }

    if poset_py.nodes.is_empty() {
        return Ok(None);
    }

    let poset = Poset::from_covering_relation(
        poset_py.nodes.clone(),
        poset_py.edges.clone(),
    )
    .map_err(|_| pyo3::exceptions::PyValueError::new_err("Edges form a cycle"))?;

    let raw_drawing = match algorithm {
        "dimdraw" => DimDraw { timeout_ms: 0 }.draw_poset(&poset),
        "sugiyama" => Sugiyama { vertex_spacing: 1 }.draw_poset(&poset),
        _ => unreachable!(),
    };

    let raw_drawing = match raw_drawing {
        Some(d) => d,
        None => return Ok(None),
    };

    let arc_labels = Arc::new(poset_py.nodes.clone());
    let coordinates = raw_drawing.coordinates.clone();
    let edges = poset.covering_edges.clone();

    let nodes: Vec<DrawingNode> = raw_drawing
        .coordinates
        .iter()
        .enumerate()
        .map(|(i, &(x, y))| {
            let mut bits = BitSet::new();
            bits.insert(i);
            let extent = LabelSet {
                bits: bits.clone(),
                labels: Arc::clone(&arc_labels),
            };
            let intent = LabelSet {
                bits: BitSet::new(),
                labels: Arc::clone(&arc_labels),
            };
            DrawingNode {
                index: i,
                x,
                y,
                concept: Concept { extent, intent },
                object_labels: vec![poset_py.nodes[i].clone()],
                attribute_labels: vec![],
            }
        })
        .collect();

    Ok(Some(Drawing {
        coordinates,
        edges,
        nodes,
    }))
}
