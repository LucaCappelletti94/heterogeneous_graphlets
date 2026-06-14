# Heterogeneous Graphlets

[![Build status](https://github.com/lucacappelletti94/heterogeneous_graphlets/actions/workflows/rust.yml/badge.svg)](https://github.com/lucacappelletti94/heterogeneous_graphlets/actions)
[![Crates.io](https://img.shields.io/crates/v/heterogeneous_graphlets.svg)](https://crates.io/crates/heterogeneous_graphlets)
[![Documentation](https://docs.rs/heterogeneous_graphlets/badge.svg)](https://docs.rs/heterogeneous_graphlets)
[![codecov](https://codecov.io/gh/LucaCappelletti94/heterogeneous_graphlets/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/heterogeneous_graphlets)

Rust implementation of heterogeneous (typed) graphlet counting, after Rossi et al., "Heterogeneous Graphlets" (ACM TKDD 2020). Graphlets can be typed by node colour, by edge colour, or by both.

## Heterogeneous (coloured) graphlets

A graphlet is a small connected subgraph. A *heterogeneous* graphlet additionally colours its nodes by type and its edges by type. For each edge of the graph, this crate counts the 4-node graphlet orbits the edge participates in, separately for every combination of node and edge colours. There are twelve edge orbits, distinguished by where the counted edge sits inside the graphlet:

![The twelve heterogeneous-graphlet edge orbits](https://raw.githubusercontent.com/LucaCappelletti94/heterogeneous_graphlets/main/assets/graphlets/all_graphlets.svg)

Each node is filled with its colour and each edge is drawn in its colour, with the thicker edge being the one counted. The colouring drawn in each panel is only *one example*: the same orbit with a different colouring (colours may repeat) is a different heterogeneous graphlet, counted separately. That is the whole point of the crate, and it is why a single topology stands for many counts.

The caption under each panel gives, for `c` node colours and `d` edge colours, the number of distinct typed graphlets of that orbit the counter distinguishes (its edge-centric hash granularity). A 4-clique edge, for instance, has `(c^4 d^6 + 2 c^3 d^4 + c^2 d^4) / 4` distinct typed forms. These counts are verified exhaustively by the test suite.

## Two counters: node colours, or node and edge colours

The crate exposes two edge-centric counters, both keyed by a perfect hash and both running in the same edge-centric time:

- Node colours only. Implement `Graph` and `TypedGraph`, opt into `HeterogeneousGraphlets`, and call `get_heterogeneous_graphlet(src, dst)`. Graphlets are distinguished by their node colours alone.
- Node and edge colours. Additionally implement `EdgeTypedGraph` (which adds edge-colour access), opt into `EdgeTypedGraphlets`, and call `get_edge_typed_graphlet(src, dst)`. Graphlets are distinguished by both their node and edge colours.

Summing the edge-coloured output over the edge colours recovers the node-only counts exactly, so the edge-coloured counter is a strict refinement of the node-only one. Neighbour lists must be sorted in ascending order for either counter.

## How many colours fit

Each typed graphlet is stored under a perfect hash, using the integer type you pick as the `Graphlet` key (the first type parameter). The number of colours you can use is bounded by the range of that key type, and the crate checks the relevant bound on every call, returning a `GraphletError` rather than miscounting if it is exceeded.

For the node-only counter the key packs `(orbit kind, four node colours)` in base `c + 1`, so `c` is the largest value satisfying

```text
13*(c+1)^4 + (c+1)^3 + (c+1)^2 + (c+1)  <=  Graphlet::MAX
```

| `Graphlet` key | maximum node colours |
| -------------- | -------------------: |
| `u8`           |                    1 |
| `u16`          |                    7 |
| `u32`          |                  133 |
| `u64`          |               34,512 |
| `u128`         |        2,261,903,241 |

(`Cora` has 7 node colours and `CiteSeer` 6, which is why both fit `u16`.)

For the edge-coloured counter the key also packs the six edge colours in base `d + 1`, so for `c` node colours and `d` edge colours the bound is

```text
12 * (c+1)^4 * (d+1)^6  <=  Graphlet::MAX
```

The `(d+1)^6` factor grows quickly, so the edge-coloured counter usually wants a `u64` or `u128` key: a `u32` holds only up to about `(c, d) = (5, 5)`, whereas `u64` and `u128` comfortably cover the colour counts of typical attributed graphs. A second, usually looser, cap comes from the range of the `NodeLabel` and `EdgeLabel` types themselves.

## Usage

The example below implements one small graph and counts both ways: node colours only with `get_heterogeneous_graphlet`, and node plus edge colours with `get_edge_typed_graphlet`.

```rust
use heterogeneous_graphlets::prelude::*;
use hashbrown::HashMap;

// A small undirected graph stored as per-node sorted adjacency lists, with one
// colour (type) per node and a simple deterministic edge colouring.
struct AdjacencyGraph {
    neighbours: Vec<Vec<usize>>,
    labels: Vec<u8>,
    number_of_labels: u8,
    number_of_edge_labels: u8,
}

impl Graph for AdjacencyGraph {
    type NeighbourIter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn get_number_of_nodes(&self) -> usize {
        self.labels.len()
    }

    fn get_number_of_edges(&self) -> usize {
        self.neighbours.iter().map(Vec::len).sum()
    }

    fn iter_neighbours(&self, node: usize) -> Self::NeighbourIter<'_> {
        self.neighbours[node].iter().copied()
    }
}

impl TypedGraph for AdjacencyGraph {
    type NodeLabel = u8;

    fn get_number_of_node_labels(&self) -> u8 {
        self.number_of_labels
    }

    fn get_number_of_node_labels_usize(&self) -> usize {
        self.number_of_labels as usize
    }

    fn get_node_label_from_usize(&self, label_index: usize) -> u8 {
        label_index as u8
    }

    fn get_node_label_index(&self, label: u8) -> usize {
        label as usize
    }

    fn get_node_label(&self, node: usize) -> u8 {
        self.labels[node]
    }
}

// Implementing EdgeTypedGraph opts the graph into edge-coloured counting. Here
// the colour of edge (a, b) is (a + b) % number_of_edge_labels.
impl EdgeTypedGraph for AdjacencyGraph {
    type EdgeLabel = u8;

    fn get_number_of_edge_labels(&self) -> u8 {
        self.number_of_edge_labels
    }

    fn get_number_of_edge_labels_usize(&self) -> usize {
        self.number_of_edge_labels as usize
    }

    fn get_edge_label_from_usize(&self, label_index: usize) -> u8 {
        label_index as u8
    }

    fn get_edge_label_index(&self, label: u8) -> usize {
        label as usize
    }

    fn get_edge_label(&self, src: usize, dst: usize) -> u8 {
        ((src + dst) % usize::from(self.number_of_edge_labels)) as u8
    }
}

// The node-only counter: pick u32 for the perfect-hash key and u32 for the counts.
impl HeterogeneousGraphlets<u32, u32> for AdjacencyGraph {
    type GraphLetCounter = HashMap<u32, u32>;
}

// The edge-coloured counter uses a wider key, since the edge colours enlarge the
// hash (see "How many colours fit" above).
impl EdgeTypedGraphlets<u64, u64> for AdjacencyGraph {
    type GraphLetCounter = HashMap<u64, u64>;
}

// A 4-clique on nodes {0, 1, 2, 3}, all sharing the single node colour 0, with
// two edge colours.
let graph = AdjacencyGraph {
    neighbours: vec![vec![1, 2, 3], vec![0, 2, 3], vec![0, 1, 3], vec![0, 1, 2]],
    labels: vec![0, 0, 0, 0],
    number_of_labels: 1,
    number_of_edge_labels: 2,
};

// Node-coloured counts incident to the edge (0, 1). This returns an error only if
// the chosen Graphlet key type is too small for the colour count.
let node_counts = graph
    .get_heterogeneous_graphlet(0, 1)
    .expect("u32 is wide enough for a single node colour");

// Group the per-orbit counts by graphlet-kind name for inspection.
let by_kind =
    node_counts.to_graphlet_names::<ExtendedGraphletType, u8>(graph.get_number_of_node_labels());

// The edge (0, 1) of a 4-clique lies in two triangles and one 4-clique. Look the
// orbits up by their name constants rather than by spelling the string literal.
assert_eq!(by_kind.get(ExtendedGraphletType::TRIANGLE), Some(&2));
assert_eq!(by_kind.get(ExtendedGraphletType::FOUR_CLIQUE), Some(&1));

// Edge-coloured counts incident to the same edge: the same graphlets, now also
// split by edge colour. Summed over every key, the two counters agree, because
// the edge-coloured counter is a refinement of the node-only one.
let edge_counts = graph
    .get_edge_typed_graphlet(0, 1)
    .expect("u64 is wide enough for these colour counts");

assert_eq!(
    edge_counts.values().sum::<u64>(),
    u64::from(node_counts.values().sum::<u32>()),
);
```

## Reference

This crate implements the framework introduced in:

Ryan A. Rossi, Nesreen K. Ahmed, Aldo Carranza, David Arbour, Anup Rao, Sungchul Kim, and Eunyee Koh. "Heterogeneous Graphlets." ACM Transactions on Knowledge Discovery from Data (TKDD), 15(1), 2020, pp. 1-43. [doi:10.1145/3418773](https://doi.org/10.1145/3418773). Preprint: [arXiv:2010.14058](https://arxiv.org/abs/2010.14058).

```bibtex
@article{rossi2020heterogeneous,
  author  = {Rossi, Ryan A. and Ahmed, Nesreen K. and Carranza, Aldo and Arbour, David and Rao, Anup and Kim, Sungchul and Koh, Eunyee},
  title   = {Heterogeneous Graphlets},
  journal = {ACM Transactions on Knowledge Discovery from Data},
  volume  = {15},
  number  = {1},
  pages   = {1--43},
  year    = {2020},
  doi     = {10.1145/3418773},
}
```
