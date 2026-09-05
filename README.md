# odis-python

[![PyPI](https://img.shields.io/pypi/v/odis-python.svg)](https://pypi.org/project/odis-python/)

Python bindings for the **odis** Formal Concept Analysis library, powered by Rust and PyO3.

## Background

[Formal Concept Analysis](https://en.wikipedia.org/wiki/Formal_concept_analysis) (FCA)
works on formal contexts — cross-tables pairing objects with attributes via a binary
incidence relation — and derives the complete lattice of formal concepts from them.
odis implements the core FCA algorithms in Rust and exposes them through this Python
interface. For an introduction to FCA see [Uta Priss's FCA page](https://www.upriss.org.uk/fca/fca.html).

## Installation

### Released package (PyPI)

```bash
pip install odis-python
```

### Development build (from source)

Requires a Rust toolchain and [maturin](https://www.maturin.rs/).

```bash
git clone https://github.com/odis-rs/odis
cd odis/odis-python
pip install maturin
maturin develop --release
```

## Quick Start

```python
from odis import FormalContext

ctx = FormalContext.from_file("odis/test_data/living_beings_and_water.cxt")
print(f"Objects: {ctx.objects}")
print(f"Attributes: {ctx.attributes}")
concepts = list(ctx.concepts())
print(f"Number of concepts: {len(concepts)}")
```

## FormalContext

`FormalContext` stores a set of *objects*, a set of *attributes*, and a binary *incidence
relation* mapping object–attribute pairs.

### Construction

```python
from odis import FormalContext

# Empty context
ctx = FormalContext()

# From a .cxt (Burmeister) file
ctx = FormalContext.from_file("odis/test_data/living_beings_and_water.cxt")

# From a dict mapping each object to its set of attributes
animals = FormalContext.from_dict({
    "cat":  {"has_legs", "has_fur", "can_move"},
    "fish": {"lives_in_water", "can_move"},
    "fern": {"needs_chlorophyll"},
})
```

The examples below all use `ctx` as loaded from
`living_beings_and_water.cxt`, whose objects are `fish leech`, `bream`, `frog`,
`dog`, `water weeds`, `reed`, `bean` and `corn`. Note that `fish leech` is one
object — the animal — not two.

### Introspection

```python
n_objects, n_attributes = ctx.shape   # e.g. (8, 9)
n = len(ctx)                          # same as ctx.shape[0] — number of objects
print(ctx.objects)                    # ['fish leech', 'bream', 'frog', ...]
print(ctx.attributes)                 # ['needs water to live', ...]
print("frog" in ctx)                  # True — tests object membership
print(repr(ctx))                      # human-readable summary
```

### Incidence Access

```python
# Read: does object have attribute?
val = ctx["frog", "lives in water"]   # True
val = ctx["frog", "breast feeds"]     # False

# Write
ctx["frog", "lives in water"] = False
ctx["frog", "lives in water"] = True
```

### Mutation

```python
# Add an object with no attributes
ctx.add_object("shark")

# Add an object with some pre-set attributes. Re-using an existing name
# raises ValueError.
ctx.add_object("whale", {"needs water to live", "can move", "breast feeds"})

# Add a new attribute column
ctx.add_attribute("is_endangered")

# Remove
ctx.remove_object("shark")
ctx.remove_object("whale")
ctx.remove_attribute("is_endangered")

# Rename
ctx.rename_object("frog", "toad")
ctx.rename_attribute("needs water to live", "aquatic")
```

### Serialisation

```python
# Save to .cxt file
ctx.to_file("/tmp/my_context.cxt")

# Deep copy — mutations to the copy do not affect the original
copy = ctx.copy()
copy.add_object("clone_only")
assert "clone_only" not in ctx.objects
```

### Derivation Operators

```python
# Extent: the set of all objects sharing every given attribute
extent = ctx.extent(["needs water to live", "can move"])

# Intent: the set of all attributes shared by every given object.
# Names that are not in the context are dropped silently, so a typo here
# quietly computes the intent of a smaller set instead of raising.
intent = ctx.intent(["fish leech", "bream"])

# Attribute hull (closure of an attribute set under the Galois connection)
hull = ctx.attribute_hull(["needs water to live"])

# Object hull (closure of an object set)
ohull = ctx.object_hull(["frog"])

# Upper neighbor: the extent of the concept directly above the given concept
# in the lattice (the least concept with a strictly larger extent)
neighbor = ctx.upper_neighbor(["frog"])

# All results are LabelSets — iterate or convert freely
print(list(extent))        # ['bream', 'dog', 'fish leech', 'frog']
print("frog" in extent)    # True or False
```

### Drawing Shortcut

`FormalContext` provides convenience methods to draw the concept lattice without
instantiating a `Drawing` object; see [Drawing](#drawing) for the full API.

```python
svg_str = ctx.draw_svg("dimdraw", width=800, height=600)
drawing  = ctx.draw("dimdraw")
```

## Concepts

`FormalContext.concepts()` returns a `ConceptCollection` (eager, indexable) or a
`ConceptGenerator` (lazy, forward-only). Each element is a `Concept` with `.extent`
and `.intent` properties.

```python
# Eager (default) — all concepts materialised at once
concepts = ctx.concepts()
print(f"Found {len(concepts)} concepts")

# Access by index
first = concepts[0]
print(list(first.extent))   # objects in this concept
print(list(first.intent))   # attributes in this concept

# Iteration with unpacking
for extent, intent in concepts:
    print(list(extent), "→", list(intent))
```

Lazy concepts are covered under [Lazy Generators & Mutation Guard](#lazy-generators--mutation-guard).

## Implications

The **canonical implication basis** (Duquenne–Guigues basis) is the smallest set of
implications that logically entails all implications valid in the context.

```python
basis = ctx.canonical_basis()
print(f"Basis size: {len(basis)}")

for impl in basis:
    print(list(impl.premise), "→", list(impl.conclusion))

# Access by index
imp = basis[0]
print(list(imp.premise))     # antecedent attributes
print(list(imp.conclusion))  # consequent attributes

# Optimised variant (same result, faster in practice)
basis_opt = ctx.canonical_basis_optimised()
```

Iterating pseudo-intents one at a time with `next_preclosure`:

```python
# next_preclosure(basis, current) returns the next closed attribute set in
# lectic order. Terminates naturally when len(result) == number of attributes.
n_attrs = len(ctx.attributes)
current = frozenset()
while len(current) < n_attrs:
    nxt = ctx.next_preclosure(basis, current)
    if len(nxt) == n_attrs:
        break
    print(list(nxt))
    current = nxt
```

## Attribute Exploration

Attribute exploration is an interactive algorithm that discovers the canonical basis
by consulting an oracle (a Python callback) about whether proposed implications hold.
The oracle may reject an implication by supplying a counterexample.

```python
def my_oracle(premise, conclusion):
    """Called for each proposed implication.

    premise and conclusion are LabelSets (iterable over strings).
    Return True to accept; return (name, attrs) to reject with a counterexample.
    """
    print(f"Does: {list(premise)} → {list(conclusion)}?")
    return True  # accept everything: the result is the canonical basis

basis = ctx.attribute_exploration(my_oracle)
print(f"Discovered {len(basis)} implications")
```

The callback receives two `LabelSet` arguments — `premise` and `conclusion`:
- Return any truthy non-tuple value (e.g. `True`) to **accept** the implication.
- Return `(name: str, attributes: Iterable[str])` to **reject** it with a counterexample.

When a counterexample is provided, `attribute_exploration` adds that object (with the
given attributes) to the context and continues.

A counterexample has to be one: the object must have **every** attribute of the
premise and **miss at least one** of the conclusion. An object that does not
refute the implication leaves it valid, so it is proposed again — and an oracle
that keeps answering with the same object never terminates. Deriving the
counterexample from the premise itself is always safe:

```python
rejected = 0

def counterexample_oracle(premise, conclusion):
    global rejected
    if "can move" in list(premise):
        rejected += 1
        # Has exactly the premise, so it misses the conclusion by construction.
        return (f"counterexample_{rejected}", set(premise))
    return True

ctx_copy = ctx.copy()
basis = ctx_copy.attribute_exploration(counterexample_oracle)
print(f"{rejected} rejected, {len(basis)} implications, {len(ctx_copy.objects)} objects")
```

## Drawing

`Poset` lets you directly define a partial order. Edges describe the **covering relation**:
`(u, v)` means node `u` is directly below node `v` (u ≺ v), given
as 0-based indices into the node list. Cycles are rejected with `ValueError`.

```python
from odis import Poset

# Diamond lattice
p = Poset(
    ["bottom", "left", "right", "top"],
    [(0, 1), (0, 2), (1, 3), (2, 3)],
)

# Quick SVG
svg = p.draw_svg("dimdraw", width=800, height=600)
with open("order.svg", "w") as f:
    f.write(svg)

# Drawing object for programmatic access
drawing = p.draw("dimdraw")
if drawing is not None:
    for node in drawing.nodes:
        print(f"{node.object_labels[0]}: ({node.x:.1f}, {node.y:.1f})")
    print(drawing.edges)   # list of (u, v) covering-relation pairs
```

### Concept Lattice Drawings

odis can draw the concept lattice as a directed graph. Three layout algorithms are
available: `"dimdraw"` (dimension-based, default), `"sugiyama"` (hierarchical) and
`"dimflux"` (a DimDraw layout refined into an additive one by a force-directed
model, which spreads the nodes away from the edges they are not part of).
`"dimflux"` needs the objects and attributes of a concept, so it is available on
a context but not on a bare `Poset`.

`timeout_ms` bounds the layout search and defaults to one second. Pass
`timeout_ms=None` to search until the layout is a proven optimum — the cost of
that proof climbs steeply with the size of the lattice, so it is opt-in.

```python
# Quick SVG string — no intermediate Drawing object required
svg = ctx.draw_svg("dimdraw", width=800, height=600)
with open("lattice.svg", "w") as f:
    f.write(svg)
```

```python
# Full Drawing object for programmatic access
drawing = ctx.draw("dimdraw")
if drawing is not None:
    print(f"Nodes: {len(drawing.nodes)}")
    print(f"Edges: {drawing.edges}")              # list of (from_idx, to_idx) tuples
    print(f"Coordinates: {drawing.coordinates}")  # raw layout (x, y) per node

    for node in drawing.nodes:
        print(f"  node {node.index}: ({node.x:.1f}, {node.y:.1f})")
        print(f"    reduced objects:    {node.object_labels}")
        print(f"    reduced attributes: {node.attribute_labels}")

    # Convert to SVG from Drawing object (useful for custom sizes)
    svg2 = drawing.to_svg(ctx, width=1200, height=800)
    with open("large_lattice.svg", "w") as f:
        f.write(svg2)

# Jupyter notebook: display inline (requires IPython)
try:
    from IPython.display import SVG, display
    display(SVG(data=svg))
except ImportError:
    pass  # not running in a notebook
```

### DimFlux

`"dimflux"` starts from the DimDraw layout and projects it into the space of
*additive* diagrams, where every concept sits at the sum of one vector per
object in its extent and per attribute in its intent. A force-directed model
then spreads the nodes away from the edges they are not part of, without letting
them leave the cells DimDraw put them in.

Two properties follow, and both help a reader: equal steps through the lattice
are drawn as equal vectors — so a distributive part of the lattice comes out as
a grid of parallelograms — and concept nodes keep their distance from unrelated
edges.

```python
ctx = FormalContext.from_file("odis/test_data/living_beings_and_water.cxt")

# The same lattice under each of the three layouts
for algorithm in ("dimdraw", "sugiyama", "dimflux"):
    svg = ctx.draw_svg(algorithm, width=800, height=600)
    with open(f"lattice_{algorithm}.svg", "w") as f:
        f.write(svg)

# The additive structure shows up in the coordinates: two covering edges that
# add the same objects and drop the same attributes come out as the same vector.
from collections import defaultdict

drawing = ctx.draw("dimflux")
xy = drawing.coordinates
families = defaultdict(list)
for lower, upper in drawing.edges:
    step = (round(xy[upper][0] - xy[lower][0], 6),
            round(xy[upper][1] - xy[lower][1], 6))
    families[step].append((lower, upper))

parallel = {step: edges for step, edges in families.items() if len(edges) > 1}
print(f"{len(drawing.edges)} edges drawn as {len(families)} distinct vectors")
print(f"{len(parallel)} of those vectors are shared by more than one edge")
```

`"dimflux"` spends its search budget on the DimDraw layout it starts from, so
`timeout_ms` trades quality against time just as it does for `"dimdraw"`:

```python
quick = ctx.draw("dimflux", timeout_ms=100)     # good enough to look at
better = ctx.draw("dimflux", timeout_ms=5000)   # a longer search for the base layout
```

Because it needs the objects and attributes behind each node, `"dimflux"` is
available on a `FormalContext` but not on a bare `Poset`, which carries only the
order.

## Titanic

The `Titanic` algorithm enumerates *iceberg concepts* — concepts whose extent meets
a minimum support threshold. Useful for large or sparse contexts where only frequent
concepts are of interest.

```python
from odis import FormalContext, Titanic

ctx = FormalContext.from_dict({
    "a": {"x", "y", "z"},
    "b": {"x", "y"},
    "c": {"x", "z"},
    "d": {"y", "z"},
    "e": {"x"},
})

iceberg = Titanic()

# Only enumerate concepts with at least 2 objects in their extent
top_concepts = iceberg.enumerate(ctx, min_support=2)
print(f"Iceberg concepts (support ≥ 2): {len(top_concepts)}")
for c in top_concepts:
    print(f"  extent={list(c.extent)}, intent={list(c.intent)}")
```

## LabelSet

`LabelSet` is a set-like view of string labels. It is returned by derivation
operators (`extent`, `intent`, `attribute_hull`, `object_hull`, `upper_neighbor`),
implication properties (`premise`, `conclusion`), and concept properties
(`.extent`, `.intent`).

```python
intent = ctx.intent(["fish", "leech"])

# Membership test
print("can move" in intent)   # True

# Iteration — yields strings directly, no index translation needed
for attr in intent:
    print(attr)

# Convert to standard Python containers
as_list = list(intent)
as_set  = set(intent)
```

## Lazy Generators & Mutation Guard

Passing `lazy=True` to `concepts()` or `canonical_basis()` returns a generator that
produces one concept/implication at a time without materialising the full collection.
Lazy generators hold a shared reference to the context's internal state, so **any
mutation** while a lazy generator is alive raises `RuntimeError`.

```python
ctx = FormalContext.from_file("odis/test_data/living_beings_and_water.cxt")

# Create a lazy generator
gen = ctx.concepts(lazy=True)

# Iterating is safe
first = next(gen)
print(list(first.extent))

# Mutating while the generator is alive raises RuntimeError
try:
    ctx.add_attribute("new_attr")       # raises RuntimeError
except RuntimeError as e:
    print(f"Caught: {e}")

# Release the generator first, then mutate freely
del gen
ctx.add_attribute("new_attr")          # OK
```

The same guard applies to `canonical_basis(lazy=True)` and
`Titanic().enumerate(ctx, ..., lazy=True)`.

## Error Reference

| Exception | When raised | Example trigger |
|---|---|---|
| `FileNotFoundError` | `.cxt` file path does not exist | `FormalContext.from_file("missing.cxt")` |
| `OSError` | Other I/O error reading a file | Unreadable file permissions |
| `ValueError` | Malformed `.cxt` file | Invalid Burmeister format |
| `KeyError` | Unknown object or attribute name | `ctx["ghost", "flies"]` |
| `ValueError` | Duplicate object or attribute name | `ctx.add_object("frog")` when already present |
| `RuntimeError` | Mutation while a lazy generator is alive | `ctx.add_attribute("x")` during active generator |
| `ValueError` | Unknown drawing algorithm | `ctx.draw("unknown_algo")` |
| `ValueError` | Non-positive SVG dimensions | `ctx.draw_svg("dimdraw", -1, 600)` |
