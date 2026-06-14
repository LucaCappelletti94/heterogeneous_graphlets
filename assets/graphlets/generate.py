#!/usr/bin/env python3
"""Generate clean, consistent SVG illustrations of the twelve heterogeneous-graphlet edge orbits.

Each orbit is a small graphlet (3 or 4 nodes) plus one distinguished edge whose
position within the graphlet defines the orbit. These are heterogeneous
graphlets: every node carries a colour (its type), which is the defining feature
of the crate, so each node is filled with its type colour from a categorical
palette. The distinguished orbit edge (the edge being counted) is drawn thicker
in a type-neutral near-black, and its two endpoint nodes get a matching dark
ring, so the highlight never competes with the node-type colours. All twelve
share the same node radius, stroke widths, palette, and font.

Run with: uv run python3 assets/graphlets/generate.py

The committed artifacts are the .svg files; this script exists to keep them
consistent and reproducible.
"""

from __future__ import annotations

import math
import os
import random
import re
import shutil
import subprocess
import tempfile
from xml.sax.saxutils import escape

# ---------------------------------------------------------------------------
# Shared visual style
# ---------------------------------------------------------------------------

CELL_W = 240  # logical width of a single graphlet panel
CELL_H = 262  # logical height of a single graphlet panel (room for name + formula)
NODE_R = 14  # node radius
NODE_R_ORBIT = 15  # slightly larger radius for the counted edge's endpoints
EDGE_W = 4.5  # ordinary edge stroke width
ORBIT_EDGE_W = 7.5  # counted orbit edge stroke width
NODE_STROKE_W = 1.5  # thin definition outline on the colour-filled node
NODE_STROKE_OPACITY = 0.35  # ordinary node outline is faint
NODE_STROKE_W_ORBIT = 3.5  # bold ink ring marks the counted edge's endpoints
ORDINARY_EDGE_OPACITY = 0.5  # ordinary edges are softened so the orbit edge reads

# "Paper" palette matching the PubChem Molecular Topology Explorer
# (topology.earthmetabolome.org): a warm paper background, ink line-art, and
# paper-filled nodes whose OUTLINE colour carries the node type.
COL_PAPER_TOP = "#efe6d7"  # background gradient stops
COL_PAPER_MID = "#f7f2e8"
COL_PAPER_BOTTOM = "#f4eee3"
COL_CARD = "#fffdf8"  # paper-strong: panel + node fill
COL_INK = "#1f2624"  # edges and captions
COL_LINE = "#ddd4c3"  # soft panel border

# Node-type ("colour") palette: each node is FILLED with its type colour, the
# defining feature of heterogeneous graphlets. The Okabe-Ito categorical
# palette gives maximally distinct hues that remain distinguishable under all
# common forms of colour-vision deficiency.
TYPE_PALETTE = [
    "#0072B2",  # type 0: blue
    "#D55E00",  # type 1: vermillion
    "#009E73",  # type 2: bluish green
    "#CC79A7",  # type 3: reddish purple
]

# Each catalog panel gets its own example colouring, drawn at random from the
# whole palette (with a fixed seed so the committed SVGs stay reproducible). The
# panels therefore show a spread of colour combinations - distinct, repeated, and
# across all four colours - to make the colour variance plain rather than
# implying a single fixed pattern.
CATALOG_COLOUR_SEED = 20240614

FONT = "font-family='Iowan Old Style, Palatino Linotype, Book Antiqua, Georgia, serif'"

# Layout area inside a cell reserved for the drawing (above the caption).
DRAW_TOP = 18
DRAW_BOTTOM = 196
CAPTION_Y = 212  # baseline of the orbit name
FORMULA_Y = 226  # top of the embedded count formula
FORMULA_H = 22  # rendered height of the count formula

# LaTeX bodies for the per-orbit count of distinct typed graphlets the algorithm
# distinguishes (its edge-centric hash granularity), as a function of the number
# of colours c. Verified exhaustively by the `edge_centric_typed_key_counts_match_formula`
# test in src/oracle.rs.
FORMULA_CUBE = r"$c^{3}$"
FORMULA_FOURTH = r"$c^{4}$"
FORMULA_HALF = r"$\sfrac{c^{3}(c+1)}{2}$"


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
# (index, file stem, caption, builder, count-formula LaTeX body).
ORBITS = [
    (0, "triad", "Triad", triad, FORMULA_CUBE),
    (1, "triangle", "Triangle", triangle, FORMULA_CUBE),
    (2, "four_path_edge", "FourPathEdge", four_path_edge, FORMULA_FOURTH),
    (3, "four_path_center", "FourPathCenter", four_path_center, FORMULA_HALF),
    (4, "four_star", "FourStar", four_star, FORMULA_HALF),
    (5, "four_cycle", "FourCycle", four_cycle, FORMULA_HALF),
    (6, "tailed_tri_tail", "TailedTriTail", tailed_tri_tail, FORMULA_HALF),
    (7, "tailed_tri_center", "TailedTriCenter", tailed_tri_center, FORMULA_FOURTH),
    (8, "tailed_tri_edge", "TailedTriEdge", tailed_tri_edge, FORMULA_HALF),
    (9, "chordal_cycle_edge", "ChordalCycleEdge", chordal_cycle_edge, FORMULA_HALF),
    (10, "chordal_cycle_center", "ChordalCycleCenter", chordal_cycle_center, FORMULA_HALF),
    (11, "four_clique", "FourClique", four_clique, FORMULA_HALF),
]


def catalog_colourings() -> dict:
    """An example node colouring per orbit, drawn from the whole palette with a
    fixed seed (so the committed SVGs are reproducible).

    Each panel uses at least two colours, and the colourings vary across panels
    (distinct, repeated, spanning all four colours), so the figure makes the
    colour variance plain instead of suggesting a single fixed pattern.
    """
    rng = random.Random(CATALOG_COLOUR_SEED)
    n_colours = len(TYPE_PALETTE)
    colourings: dict[int, list[int]] = {}
    for idx, _stem, _caption, builder, _body in ORBITS:
        num_nodes = len(builder()["nodes"])
        while True:
            choice = [rng.randrange(n_colours) for _ in range(num_nodes)]
            if len(set(choice)) >= 2:  # avoid a monochrome (no-variance) panel
                break
        colourings[idx] = choice
    return colourings


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------


def _orbit_match(edge: tuple[str, str], orbit: tuple[str, str]) -> bool:
    return set(edge) == set(orbit)


def render_graphlet(
    spec: dict,
    ox: float,
    oy: float,
    indent: str = "  ",
    type_indices: list[int] | None = None,
) -> str:
    """Render a single graphlet's edges and nodes, translated by (ox, oy).

    By default each node is filled with a distinct type colour
    (``index % len(TYPE_PALETTE)``). Pass ``type_indices`` (one entry per node,
    in node-insertion order) to assign types explicitly, which lets a panel
    REPEAT a colour: a heterogeneous graphlet's node colours need not differ.
    """
    nodes = spec["nodes"]
    edges = spec["edges"]
    orbit = spec["orbit"]
    orbit_endpoints = set(orbit)

    parts: list[str] = []

    # Edges first (so nodes sit on top). Ordinary edges, then the orbit edge
    # last so it renders above any crossing line.
    ordinary = [e for e in edges if not _orbit_match(e, orbit)]
    distinguished = [e for e in edges if _orbit_match(e, orbit)]

    # Ordinary edges are softened ink; the counted orbit edge is full-strength
    # ink and thicker, drawn last so it sits above any crossing line.
    for u, v in ordinary:
        x1, y1 = nodes[u]
        x2, y2 = nodes[v]
        parts.append(
            f"{indent}<line x1='{x1 + ox:.1f}' y1='{y1 + oy:.1f}' "
            f"x2='{x2 + ox:.1f}' y2='{y2 + oy:.1f}' "
            f"stroke='{COL_INK}' stroke-opacity='{ORDINARY_EDGE_OPACITY}' "
            f"stroke-width='{EDGE_W}' stroke-linecap='round'/>"
        )
    for u, v in distinguished:
        x1, y1 = nodes[u]
        x2, y2 = nodes[v]
        parts.append(
            f"{indent}<line x1='{x1 + ox:.1f}' y1='{y1 + oy:.1f}' "
            f"x2='{x2 + ox:.1f}' y2='{y2 + oy:.1f}' "
            f"stroke='{COL_INK}' stroke-width='{ORBIT_EDGE_W}' "
            f"stroke-linecap='round'/>"
        )

    # Nodes are FILLED with their type colour (the defining feature). Ordinary
    # nodes get a faint ink outline for definition; the two endpoints of the
    # counted edge get a bold ink ring and a larger radius so the orbit reads
    # clearly, in ink rather than colour so it never competes with the types.
    for index, (name, (x, y)) in enumerate(nodes.items()):
        if type_indices is not None:
            type_colour = TYPE_PALETTE[type_indices[index] % len(TYPE_PALETTE)]
        else:
            type_colour = TYPE_PALETTE[index % len(TYPE_PALETTE)]
        if name in orbit_endpoints:
            radius, stroke_w, stroke_op = NODE_R_ORBIT, NODE_STROKE_W_ORBIT, 1.0
        else:
            radius, stroke_w, stroke_op = NODE_R, NODE_STROKE_W, NODE_STROKE_OPACITY
        parts.append(
            f"{indent}<circle cx='{x + ox:.1f}' cy='{y + oy:.1f}' r='{radius}' "
            f"fill='{type_colour}' stroke='{COL_INK}' stroke-width='{stroke_w}' "
            f"stroke-opacity='{stroke_op}'/>"
        )

    return "\n".join(parts)


def _paper_gradient_def(grad_id: str, height: float) -> str:
    """A vertical warm-paper gradient matching the reference site background."""
    return (
        f"  <defs>\n"
        f"    <linearGradient id='{grad_id}' x1='0' y1='0' x2='0' y2='{height:.0f}' "
        f"gradientUnits='userSpaceOnUse'>\n"
        f"      <stop offset='0' stop-color='{COL_PAPER_TOP}'/>\n"
        f"      <stop offset='0.44' stop-color='{COL_PAPER_MID}'/>\n"
        f"      <stop offset='1' stop-color='{COL_PAPER_BOTTOM}'/>\n"
        f"    </linearGradient>\n"
        f"  </defs>\n"
    )


def standalone_svg(
    spec: dict, index: int, caption: str, formula: dict, type_indices: list[int]
) -> str:
    title = f"{caption} (orbit {index})"
    body = render_graphlet(spec, 0, 0, indent="  ", type_indices=type_indices)
    caption_text = escape(caption)
    count = _embed_latex(formula, CX, FORMULA_Y, FORMULA_H, id_prefix=f"f{index}_")
    return (
        f"<svg xmlns='http://www.w3.org/2000/svg' "
        f"xmlns:xlink='http://www.w3.org/1999/xlink' "
        f"viewBox='0 0 {CELL_W} {CELL_H}' width='{CELL_W}' height='{CELL_H}' "
        f"role='img' aria-label='{escape(title)}'>\n"
        f"{_paper_gradient_def('paper', CELL_H)}"
        f"  <rect width='{CELL_W}' height='{CELL_H}' rx='16' fill='url(#paper)' "
        f"stroke='{COL_LINE}' stroke-width='1'/>\n"
        f"  <title>{escape(title)}</title>\n"
        f"{body}\n"
        f"  <text x='{CX}' y='{CAPTION_Y}' text-anchor='middle' "
        f"{FONT} font-size='18' font-weight='600' fill='{COL_INK}'>"
        f"{caption_text}</text>\n"
        f"{count}\n"
        f"</svg>\n"
    )


def _legend(total_w: float, y: float, indent: str = "  ") -> str:
    """A centred legend explaining the colour and orbit-edge conventions."""
    parts: list[str] = []
    # Two groups: the counted-edge swatch and the node-type swatches. Their
    # combined width is centred within total_w.
    edge_len = 42
    edge_label = "Counted edge orbit"
    type_label = "Node fill = node type (colour)"
    dot_r = 9
    dot_gap = 26
    group_gap = 70
    # Rough text widths at 18px for centring (monospace-ish estimate).
    edge_text_w = 8.0 * len(edge_label)
    type_text_w = 8.0 * len(type_label)
    group_a_w = edge_len + 12 + edge_text_w
    group_b_w = 3 * dot_gap + 2 * dot_r + 12 + type_text_w
    total = group_a_w + group_gap + group_b_w
    x = (total_w - total) / 2.0

    # Group A: orbit-edge swatch (ink line).
    parts.append(
        f"{indent}<line x1='{x:.1f}' y1='{y:.1f}' x2='{x + edge_len:.1f}' y2='{y:.1f}' "
        f"stroke='{COL_INK}' stroke-width='{ORBIT_EDGE_W}' stroke-linecap='round'/>"
    )
    tx = x + edge_len + 12
    parts.append(
        f"{indent}<text x='{tx:.1f}' y='{y + 6:.1f}' {FONT} font-size='18' "
        f"fill='{COL_INK}'>{escape(edge_label)}</text>"
    )

    # Group B: node-type swatches (filled with the type colour).
    bx = x + group_a_w + group_gap
    for k in range(4):
        parts.append(
            f"{indent}<circle cx='{bx + k * dot_gap:.1f}' cy='{y:.1f}' r='{dot_r}' "
            f"fill='{TYPE_PALETTE[k]}' stroke='{COL_INK}' stroke-width='1.5' "
            f"stroke-opacity='0.35'/>"
        )
    tx2 = bx + 3 * dot_gap + dot_r + 12
    parts.append(
        f"{indent}<text x='{tx2:.1f}' y='{y + 6:.1f}' {FONT} font-size='18' "
        f"fill='{COL_INK}'>{escape(type_label)}</text>"
    )
    return "\n".join(parts)


def composed_svg(formulas: dict, colourings: dict, cols: int = 4, rows: int = 3) -> str:
    pad = 10
    legend_h = 104
    total_w = cols * CELL_W + (cols + 1) * pad
    total_h = rows * CELL_H + (rows + 1) * pad + legend_h
    parts: list[str] = [
        f"<svg xmlns='http://www.w3.org/2000/svg' "
        f"xmlns:xlink='http://www.w3.org/1999/xlink' "
        f"viewBox='0 0 {total_w} {total_h}' width='{total_w}' height='{total_h}' "
        f"role='img' aria-label='The twelve heterogeneous-graphlet edge orbits'>",
        "  <title>The twelve heterogeneous-graphlet edge orbits</title>",
        f"{_paper_gradient_def('paper', total_h)}"
        f"  <defs>\n"
        f"    <filter id='cardShadow' x='-20%' y='-20%' width='140%' height='140%'>\n"
        f"      <feDropShadow dx='0' dy='6' stdDeviation='10' "
        f"flood-color='#332d1c' flood-opacity='0.10'/>\n"
        f"    </filter>\n"
        f"  </defs>",
        f"  <rect width='{total_w}' height='{total_h}' fill='url(#paper)'/>",
    ]
    for idx, stem, caption, builder, _formula_body in ORBITS:
        col = idx % cols
        row = idx // cols
        ox = pad + col * (CELL_W + pad)
        oy = pad + row * (CELL_H + pad)
        spec = builder()
        # Paper card with a soft shadow, matching the reference site panels.
        parts.append(
            f"  <rect x='{ox}' y='{oy}' width='{CELL_W}' height='{CELL_H}' "
            f"fill='{COL_CARD}' stroke='{COL_LINE}' stroke-width='1' rx='16' "
            f"filter='url(#cardShadow)'/>"
        )
        # Each panel uses its own example colouring (see catalog_colourings),
        # drawn from the whole palette so the colour variance is visible.
        parts.append(
            render_graphlet(spec, ox, oy, indent="  ", type_indices=colourings[idx])
        )
        parts.append(
            f"  <text x='{ox + CX}' y='{oy + CAPTION_Y}' text-anchor='middle' "
            f"{FONT} font-size='18' font-weight='600' fill='{COL_INK}'>"
            f"{escape(caption)}</text>"
        )
        # The count of distinct typed graphlets of this orbit the algorithm
        # distinguishes for c colours, rendered from LaTeX.
        parts.append(
            _embed_latex(
                formulas[idx], ox + CX, oy + FORMULA_Y, FORMULA_H, id_prefix=f"f{idx}_"
            )
        )
    grid_bottom = rows * CELL_H + (rows + 1) * pad
    parts.append(_legend(total_w, grid_bottom + 24))
    # A short serif note explaining the per-panel count formulas.
    note_lines = [
        "Each caption gives the number of distinct typed graphlets of that orbit "
        "the counter distinguishes for c node colours;",
        "the colouring drawn is just one of them (colours may repeat).",
    ]
    for li, line in enumerate(note_lines):
        parts.append(
            f"  <text x='{total_w / 2.0:.1f}' y='{grid_bottom + 60 + li * 22:.1f}' "
            f"text-anchor='middle' {FONT} font-size='15' fill='{COL_INK}' "
            f"fill-opacity='0.85'>{escape(line)}</text>"
        )
    parts.append("</svg>\n")
    return "\n".join(parts)


# ---------------------------------------------------------------------------
# LaTeX -> SVG (for the typed-variant count formula)
#
# The committed SVGs must be self-contained, so we render the maths once with
# latex + dvisvgm and INLINE the result as a nested <svg> (glyphs as vector
# paths, ids prefixed to avoid collisions, wrapped in an ink-coloured group).
# ---------------------------------------------------------------------------


def _render_latex_math(latex_body: str, id_prefix: str) -> dict:
    """Render a LaTeX fragment to an inline-ready SVG snippet.

    Returns a dict with the inner SVG markup (``defs`` + glyph ``use`` group,
    ids prefixed), the natural ``width``/``height`` in pt, and the source
    ``viewBox``. Requires ``latex`` and ``dvisvgm`` on PATH. Temporary build
    files are created in a throwaway directory and never touch the repo tree.
    """
    if shutil.which("latex") is None or shutil.which("dvisvgm") is None:
        raise RuntimeError(
            "latex and dvisvgm are required to render the formula; install a "
            "TeX distribution (e.g. texlive) with dvisvgm."
        )
    tex = (
        "\\documentclass[border=2pt,varwidth]{standalone}\n"
        "\\usepackage{amsmath}\n"
        "\\usepackage{xfrac}\n"
        "\\begin{document}\n"
        f"{latex_body}\n"
        "\\end{document}\n"
    )
    with tempfile.TemporaryDirectory() as tmp:
        tex_path = os.path.join(tmp, "formula.tex")
        with open(tex_path, "w", encoding="utf-8") as fh:
            fh.write(tex)
        subprocess.run(
            ["latex", "-interaction=nonstopmode", "-halt-on-error", "formula.tex"],
            cwd=tmp,
            check=True,
            capture_output=True,
        )
        subprocess.run(
            ["dvisvgm", "--no-fonts", "--exact-bbox", "-o", "formula.svg", "formula.dvi"],
            cwd=tmp,
            check=True,
            capture_output=True,
        )
        with open(os.path.join(tmp, "formula.svg"), "r", encoding="utf-8") as fh:
            raw = fh.read()

    # Pull width / height / viewBox off the dvisvgm root <svg>.
    width = float(re.search(r"width='([\d.]+)pt'", raw).group(1))
    height = float(re.search(r"height='([\d.]+)pt'", raw).group(1))
    view_box = re.search(r"viewBox='([^']+)'", raw).group(1)

    # Keep only the <defs>...</defs> and the <g id='page1'>...</g> body.
    defs = re.search(r"<defs>.*?</defs>", raw, re.DOTALL).group(0)
    body = re.search(r"<g id='page1'>.*?</g>", raw, re.DOTALL).group(0)
    inner = f"{defs}\n{body}"

    # Prefix every glyph id (and matching xlink:href) so multiple embeds and the
    # host document never collide.
    inner = re.sub(r"id='(g\d+-[\w-]+)'", rf"id='{id_prefix}\1'", inner)
    inner = re.sub(
        r"xlink:href='#(g\d+-[\w-]+)'", rf"xlink:href='#{id_prefix}\1'", inner
    )
    # Rename the page group id too.
    inner = inner.replace("<g id='page1'>", f"<g id='{id_prefix}page'>")

    return {"inner": inner, "width": width, "height": height, "view_box": view_box}


def _embed_latex(
    rendered: dict, x: float, y: float, target_h: float, id_prefix: str
) -> str:
    """Wrap a rendered-LaTeX snippet as a nested <svg> placed at (x, y).

    The snippet is scaled so its natural height becomes ``target_h`` (in user
    units), horizontally centred on ``x``, and coloured in the ink colour.
    """
    scale = target_h / rendered["height"]
    w = rendered["width"] * scale
    h = target_h
    return (
        f"  <svg x='{x - w / 2.0:.2f}' y='{y:.2f}' width='{w:.2f}' height='{h:.2f}' "
        f"viewBox='{rendered['view_box']}' "
        f"xmlns:xlink='http://www.w3.org/1999/xlink' overflow='visible'>\n"
        f"    <g fill='{COL_INK}'>\n"
        f"{rendered['inner']}\n"
        f"    </g>\n"
        f"  </svg>"
    )


def main() -> None:
    here = os.path.dirname(os.path.abspath(__file__))

    # Render each orbit's count formula once (per-orbit id prefix so the inlined
    # glyph ids never collide inside the composed figure).
    formulas = {
        idx: _render_latex_math(body, id_prefix=f"f{idx}_")
        for idx, _stem, _caption, _builder, body in ORBITS
    }
    colourings = catalog_colourings()

    for idx, stem, caption, builder, _body in ORBITS:
        spec = builder()
        svg = standalone_svg(spec, idx, caption, formulas[idx], colourings[idx])
        path = os.path.join(here, f"{idx:02d}_{stem}.svg")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(svg)
        print(f"wrote {path}")

    composed = composed_svg(formulas, colourings, cols=4, rows=3)
    path = os.path.join(here, "all_graphlets.svg")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(composed)
    print(f"wrote {path}")


if __name__ == "__main__":
    main()
