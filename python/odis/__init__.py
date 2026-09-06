"""Formal Concept Analysis in Rust, for Python.

The heavy lifting happens in the compiled extension :mod:`odis._odis`; this
module re-exports its contents so that ``import odis`` is the only entry point
users need.
"""

from collections.abc import Set as _AbcSet

from ._odis import (
    Concept,
    ConceptCollection,
    ConceptGenerator,
    ConceptIterator,
    ConceptPairIterator,
    Drawing,
    DrawingNode,
    FormalContext,
    Implication,
    ImplicationGenerator,
    ImplicationIterator,
    ImplicationList,
    ImplicationPairIterator,
    LabelSet,
    LabelSetIterator,
    Poset,
    RepositoryEntry,
    Titanic,
    repository_catalog,
)

# LabelSet is set-like; registering it makes isinstance(x, collections.abc.Set)
# true, which is what duck-typed code checks for.
_AbcSet.register(LabelSet)

__version__ = "2026.9.1"

__all__ = [
    "Concept",
    "ConceptCollection",
    "ConceptGenerator",
    "ConceptIterator",
    "ConceptPairIterator",
    "Drawing",
    "DrawingNode",
    "FormalContext",
    "Implication",
    "ImplicationGenerator",
    "ImplicationIterator",
    "ImplicationList",
    "ImplicationPairIterator",
    "LabelSet",
    "LabelSetIterator",
    "Poset",
    "RepositoryEntry",
    "Titanic",
    "repository_catalog",
    "__version__",
]
