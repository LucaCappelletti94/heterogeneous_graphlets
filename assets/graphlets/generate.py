#!/usr/bin/env python3
"""Generate clean, consistent SVG illustrations of the twelve heterogeneous-graphlet edge orbits.

Each orbit is a small graphlet (3 or 4 nodes) plus one distinguished edge whose
position within the graphlet defines the orbit. The distinguished orbit edge is
drawn thicker and in an accent colour, and its two endpoint nodes share that
accent colour, while the other nodes stay neutral. All twelve share the same
node radius, stroke widths, palette, and font.

Run with: uv run python3 assets/graphlets/generate.py

The committed artifacts are the .svg files; this script exists to keep them
consistent and reproducible.
"""

from __future__ import annotations

import math
import os
from xml.sax.saxutils import escape

# ---------------------------------------------------------------------------
# Shared visual style
# ---------------------------------------------------------------------------

CELL_W = 240  # logical width of a single graphlet panel
CELL_H = 240  # logical height of a single graphlet panel
NODE_R = 13  # node radius
EDGE_W = 4  # normal edge stroke width
ORBIT_EDGE_W = 8  # distinguished orbit edge stroke width

COL_BG = "#ffffff"
COL_EDGE = "#9aa0a6"  # medium grey for ordinary edges
COL_NODE = "#1f2a44"  # dark navy for ordinary nodes
COL_NODE_STROKE = "#ffffff"  # white halo around every node
COL_ACCENT = "#e8590c"  # strong orange for the orbit edge + its endpoints
COL_TEXT = "#1f2a44"  # caption colour

FONT = "font-family='Segoe UI, Helvetica, Arial, sans-serif'"

# Layout area inside a cell reserved for the drawing (above the caption).
DRAW_TOP = 18
DRAW_BOTTOM = 196
CAPTION_Y = 222


def _pt(cx: float, cy: float, r: float, angle_deg: float) -> tuple[float, float]:
    a = math.radians(angle_deg)
    return (cx + r * math.cos(a), cy + r * math.sin(a))


# ---------------------------------------------------------------------------
# Orbit definitions
#
# Each orbit specifies node positions (in a 240x240 cell, drawing area roughly
# y in [DRAW_TOP, DRAW_BOTTOM]), the list of edges, and which edge is the
# distinguished orbit edge.
# ---------------------------------------------------------------------------

# Geometry helpers for the cell.
CX = CELL_W / 2.0
MIDY = (DRAW_TOP + DRAW_BOTTOM) / 2.0  # ~107


def triad() -> dict:
    # Open wedge a-b-c, edges a-b, b-c (no a-c). Orbit edge a-b.
    a = (55, 60)
    b = (CX, 170)
    c = (185, 60)
    return {
        "nodes": {"a": a, "b": b, "c": c},
        "edges": [("a", "b"), ("b", "c")],
        "orbit": ("a", "b"),
    }


def triangle() -> dict:
    a = (55, 60)
    b = (185, 60)
    c = (CX, 175)
    return {
        "nodes": {"a": a, "b": b, "c": c},
        "edges": [("a", "b"), ("b", "c"), ("a", "c")],
        "orbit": ("a", "b"),
    }


def four_path_edge() -> dict:
    # Path a-b-c-d. Orbit edge: end edge a-b.
    y = MIDY
    a = (40, y)
    b = (107, y)
    c = (174, y)
    d = (200, y)
    # Slight zig-zag so the path reads as a path, not a straight line collision.
    a = (40, 70)
    b = (97, 150)
    c = (143, 70)
    d = (200, 150)
    return {
        "nodes": {"a": a, "b": b, "c": c, "d": d},
        "edges": [("a", "b"), ("b", "c"), ("c", "d")],
        "orbit": ("a", "b"),
    }


def four_path_center() -> dict:
    a = (40, 70)
    b = (97, 150)
    c = (143, 70)
    d = (200, 150)
    return {
        "nodes": {"a": a, "b": b, "c": c, "d": d},
        "edges": [("a", "b"), ("b", "c"), ("c", "d")],
        "orbit": ("b", "c"),
    }


def four_star() -> dict:
    # Centre s, leaves x, y, z. Orbit edge: spoke s-x.
    s = (CX, MIDY)
    x = _pt(CX, MIDY, 78, -90)  # top
    y = _pt(CX, MIDY, 78, 30)  # bottom-right
    z = _pt(CX, MIDY, 78, 150)  # bottom-left
    return {
        "nodes": {"s": s, "x": x, "y": y, "z": z},
        "edges": [("s", "x"), ("s", "y"), ("s", "z")],
        "orbit": ("s", "x"),
    }


def four_cycle() -> dict:
    # Square a-b-c-d-a. Orbit edge a-b (top edge).
    a = (60, 62)
    b = (180, 62)
    c = (180, 168)
    d = (60, 168)
    return {
        "nodes": {"a": a, "b": b, "c": c, "d": d},
        "edges": [("a", "b"), ("b", "c"), ("c", "d"), ("d", "a")],
        "orbit": ("a", "b"),
    }


def _paw_nodes() -> dict:
    # Triangle {a, b, c} with tail c-d. Node c is the tail-attachment node.
    a = (52, 58)
    b = (52, 158)
    c = (132, 108)
    d = (205, 108)
    return {"a": a, "b": b, "c": c, "d": d}


def tailed_tri_tail() -> dict:
    # Orbit edge: the tail/pendant edge c-d.
    return {
        "nodes": _paw_nodes(),
        "edges": [("a", "b"), ("b", "c"), ("a", "c"), ("c", "d")],
        "orbit": ("c", "d"),
    }


def tailed_tri_center() -> dict:
    # Orbit edge: triangle edge opposite the tail = a-b (not incident to c).
    return {
        "nodes": _paw_nodes(),
        "edges": [("a", "b"), ("b", "c"), ("a", "c"), ("c", "d")],
        "orbit": ("a", "b"),
    }


def tailed_tri_edge() -> dict:
    # Orbit edge: triangle edge incident to tail-attachment node c, e.g. b-c.
    return {
        "nodes": _paw_nodes(),
        "edges": [("a", "b"), ("b", "c"), ("a", "c"), ("c", "d")],
        "orbit": ("b", "c"),
    }


def _diamond_nodes() -> dict:
    # Diamond = K4 minus edge a-c. b and d are the degree-3 nodes (joined by
    # the chord b-d); a and c are the degree-2 nodes.
    b = (CX, 56)  # top degree-3 node
    d = (CX, 170)  # bottom degree-3 node
    a = (54, 113)  # left degree-2 node
    c = (186, 113)  # right degree-2 node
    return {"a": a, "b": b, "c": c, "d": d}


def chordal_cycle_edge() -> dict:
    # Rim edge: degree-3 node to degree-2 node, e.g. a-b.
    return {
        "nodes": _diamond_nodes(),
        "edges": [("a", "b"), ("b", "c"), ("c", "d"), ("d", "a"), ("b", "d")],
        "orbit": ("a", "b"),
    }


def chordal_cycle_center() -> dict:
    # Chord edge between the two degree-3 nodes b-d.
    return {
        "nodes": _diamond_nodes(),
        "edges": [("a", "b"), ("b", "c"), ("c", "d"), ("d", "a"), ("b", "d")],
        "orbit": ("b", "d"),
    }


def four_clique() -> dict:
    # K4, all six edges. Orbit edge a-b.
    a = (60, 62)
    b = (180, 62)
    c = (180, 168)
    d = (60, 168)
    return {
        "nodes": {"a": a, "b": b, "c": c, "d": d},
        "edges": [
            ("a", "b"),
            ("a", "c"),
            ("a", "d"),
            ("b", "c"),
            ("b", "d"),
            ("c", "d"),
        ],
        "orbit": ("a", "b"),
    }


# Index -> (filename stem, caption, builder). Order matches ExtendedGraphletType
# VARIANTS in src/graphlet_set.rs.
ORBITS = [
    (0, "triad", "Triad", triad),
    (1, "triangle", "Triangle", triangle),
    (2, "four_path_edge", "FourPathEdge", four_path_edge),
    (3, "four_path_center", "FourPathCenter", four_path_center),
    (4, "four_star", "FourStar", four_star),
    (5, "four_cycle", "FourCycle", four_cycle),
    (6, "tailed_tri_tail", "TailedTriTail", tailed_tri_tail),
    (7, "tailed_tri_center", "TailedTriCenter", tailed_tri_center),
    (8, "tailed_tri_edge", "TailedTriEdge", tailed_tri_edge),
    (9, "chordal_cycle_edge", "ChordalCycleEdge", chordal_cycle_edge),
    (10, "chordal_cycle_center", "ChordalCycleCenter", chordal_cycle_center),
    (11, "four_clique", "FourClique", four_clique),
]


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


def _orbit_match(edge: tuple[str, str], orbit: tuple[str, str]) -> bool:
    return set(edge) == set(orbit)


def render_graphlet(spec: dict, ox: float, oy: float, indent: str = "  ") -> str:
    """Render a single graphlet's edges and nodes, translated by (ox, oy)."""
    nodes = spec["nodes"]
    edges = spec["edges"]
    orbit = spec["orbit"]
    orbit_endpoints = set(orbit)

    parts: list[str] = []

    # Edges first (so nodes sit on top). Ordinary edges, then the orbit edge
    # last so it renders above any crossing line.
    ordinary = [e for e in edges if not _orbit_match(e, orbit)]
    distinguished = [e for e in edges if _orbit_match(e, orbit)]

    for u, v in ordinary:
        x1, y1 = nodes[u]
        x2, y2 = nodes[v]
        parts.append(
            f"{indent}<line x1='{x1 + ox:.1f}' y1='{y1 + oy:.1f}' "
            f"x2='{x2 + ox:.1f}' y2='{y2 + oy:.1f}' "
            f"stroke='{COL_EDGE}' stroke-width='{EDGE_W}' stroke-linecap='round'/>"
        )
    for u, v in distinguished:
        x1, y1 = nodes[u]
        x2, y2 = nodes[v]
        parts.append(
            f"{indent}<line x1='{x1 + ox:.1f}' y1='{y1 + oy:.1f}' "
            f"x2='{x2 + ox:.1f}' y2='{y2 + oy:.1f}' "
            f"stroke='{COL_ACCENT}' stroke-width='{ORBIT_EDGE_W}' "
            f"stroke-linecap='round'/>"
        )

    for name, (x, y) in nodes.items():
        fill = COL_ACCENT if name in orbit_endpoints else COL_NODE
        parts.append(
            f"{indent}<circle cx='{x + ox:.1f}' cy='{y + oy:.1f}' r='{NODE_R}' "
            f"fill='{fill}' stroke='{COL_NODE_STROKE}' stroke-width='2.5'/>"
        )

    return "\n".join(parts)


def standalone_svg(spec: dict, index: int, caption: str) -> str:
    title = f"{caption} (orbit {index})"
    body = render_graphlet(spec, 0, 0, indent="  ")
    caption_text = escape(caption)
    return (
        f"<svg xmlns='http://www.w3.org/2000/svg' "
        f"viewBox='0 0 {CELL_W} {CELL_H}' width='{CELL_W}' height='{CELL_H}' "
        f"role='img' aria-label='{escape(title)}'>\n"
        f"  <title>{escape(title)}</title>\n"
        f"  <rect width='{CELL_W}' height='{CELL_H}' fill='{COL_BG}'/>\n"
        f"{body}\n"
        f"  <text x='{CX}' y='{CAPTION_Y}' text-anchor='middle' "
        f"{FONT} font-size='18' font-weight='600' fill='{COL_TEXT}'>"
        f"{caption_text}</text>\n"
        f"</svg>\n"
    )


def composed_svg(cols: int = 4, rows: int = 3) -> str:
    pad = 10
    total_w = cols * CELL_W + (cols + 1) * pad
    total_h = rows * CELL_H + (rows + 1) * pad
    parts: list[str] = [
        f"<svg xmlns='http://www.w3.org/2000/svg' "
        f"viewBox='0 0 {total_w} {total_h}' width='{total_w}' height='{total_h}' "
        f"role='img' aria-label='The twelve heterogeneous-graphlet edge orbits'>",
        "  <title>The twelve heterogeneous-graphlet edge orbits</title>",
        f"  <rect width='{total_w}' height='{total_h}' fill='{COL_BG}'/>",
    ]
    for idx, stem, caption, builder in ORBITS:
        col = idx % cols
        row = idx // cols
        ox = pad + col * (CELL_W + pad)
        oy = pad + row * (CELL_H + pad)
        spec = builder()
        # Light panel separator.
        parts.append(
            f"  <rect x='{ox}' y='{oy}' width='{CELL_W}' height='{CELL_H}' "
            f"fill='{COL_BG}' stroke='#e6e8eb' stroke-width='1' rx='8'/>"
        )
        parts.append(render_graphlet(spec, ox, oy, indent="  "))
        parts.append(
            f"  <text x='{ox + CX}' y='{oy + CAPTION_Y}' text-anchor='middle' "
            f"{FONT} font-size='18' font-weight='600' fill='{COL_TEXT}'>"
            f"{escape(caption)}</text>"
        )
    parts.append("</svg>\n")
    return "\n".join(parts)


def main() -> None:
    here = os.path.dirname(os.path.abspath(__file__))
    for idx, stem, caption, builder in ORBITS:
        spec = builder()
        svg = standalone_svg(spec, idx, caption)
        path = os.path.join(here, f"{idx:02d}_{stem}.svg")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(svg)
        print(f"wrote {path}")

    composed = composed_svg(cols=4, rows=3)
    path = os.path.join(here, "all_graphlets.svg")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(composed)
    print(f"wrote {path}")


if __name__ == "__main__":
    main()
