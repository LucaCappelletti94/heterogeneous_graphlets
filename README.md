# Heterogeneous Graphlets

[![Build status](https://github.com/lucacappelletti94/heterogeneous_graphlets/actions/workflows/rust.yml/badge.svg)](https://github.com/lucacappelletti94/heterogeneous_graphlets/actions)
[![Crates.io](https://img.shields.io/crates/v/heterogeneous_graphlets.svg)](https://crates.io/crates/heterogeneous_graphlets)
[![Documentation](https://docs.rs/heterogeneous_graphlets/badge.svg)](https://docs.rs/heterogeneous_graphlets)
[![codecov](https://codecov.io/gh/LucaCappelletti94/heterogeneous_graphlets/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94/heterogeneous_graphlets)

Rust implementation of heterogeneous (typed) graphlet counting, after Rossi et al., "Heterogeneous Graphlets" (ACM TKDD 2020).

## Heterogeneous (coloured) graphlets

A graphlet is a small connected subgraph. A *heterogeneous* graphlet additionally gives every node a colour (its type). For each edge of the graph, this crate counts the 4-node graphlet orbits the edge participates in, separately for every combination of node colours. There are twelve edge orbits, distinguished by where the counted edge sits inside the graphlet:

![The twelve heterogeneous-graphlet edge orbits](https://raw.githubusercontent.com/LucaCappelletti94/heterogeneous_graphlets/main/assets/graphlets/all_graphlets.svg)

Each node is filled with its colour, and the bold edge is the one being counted. The colouring drawn in each panel is only *one example*: the same orbit with a different colouring (colours may repeat) is a different heterogeneous graphlet, counted separately.

That is the whole point of the crate, and it is why a single topology stands for many counts. The caption under each panel gives, for `c` node colours, the number of distinct typed graphlets of that orbit the counter distinguishes (its edge-centric hash granularity). These fall into three cases:

- `Triad`, `Triangle`: `c^3`
- `FourPathEdge`, `TailedTriCenter`: `c^4`
- the remaining eight: `c^3 (c + 1) / 2`

(So a 4-clique edge, for example, has `c^3 (c + 1) / 2 = 160` distinct typed forms at `c = 4`.) These counts are verified exhaustively by the test suite.

## Number of node colours supported

Each typed graphlet is stored under a perfect hash of `(orbit kind, the four node colours)` in base `c + 1`, using the integer type you pick as the `Graphlet` key (the first type parameter of `HeterogeneousGraphlets`). The number of colours `c` you can use is the largest value satisfying

```text
13*(c+1)^4 + (c+1)^3 + (c+1)^2 + (c+1)  <=  Graphlet::MAX
```

where `Graphlet::MAX` is the maximum value of the chosen `Graphlet` type. The crate checks this bound on every call and returns a `GraphletError` rather than miscounting if it is exceeded, so pick a key type wide enough for your colour count:

| `Graphlet` key | maximum node colours |
| -------------- | -------------------: |
| `u8`           |                    1 |
| `u16`          |                    7 |
| `u32`          |                  133 |
| `u64`          |               34,512 |
| `u128`         |        2,261,903,241 |

(`Cora` has 7 node colours and `CiteSeer` 6, which is why both fit `u16`.) A second, usually looser, cap comes from the range of the `NodeLabel` type itself.

## Usage

Implement `Graph` and `TypedGraph` for your graph type, opt into `HeterogeneousGraphlets` by choosing the integer types for the perfect-hash key and the counts, then call `get_heterogeneous_graphlet(src, dst)` on an edge. Neighbour lists must be sorted in ascending order.

```rust
use heterogeneous_graphlets::prelude::*;
use hashbrown::HashMap;

// A small undirected graph stored as per-node sorted adjacency lists, with
// one label (type) per node.
struct AdjacencyGraph {
    neighbours: Vec<Vec<usize>>,
    labels: Vec<u8>,
    number_of_labels: u8,
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

// Pick `u32` for the perfect-hash key and `u32` for the counts, backed by a
// `hashbrown::HashMap` accumulator.
impl HeterogeneousGraphlets<u32, u32> for AdjacencyGraph {
    type GraphLetCounter = HashMap<u32, u32>;
}

// A 4-clique on nodes {0, 1, 2, 3}, all sharing the single node label 0.
let graph = AdjacencyGraph {
    neighbours: vec![vec![1, 2, 3], vec![0, 2, 3], vec![0, 1, 3], vec![0, 1, 2]],
    labels: vec![0, 0, 0, 0],
    number_of_labels: 1,
};

// Count the typed graphlet orbits incident to the edge (0, 1). This returns an
// error only if the chosen `Graphlet` key type is too small for the colour count.
let counts = graph
    .get_heterogeneous_graphlet(0, 1)
    .expect("u32 is wide enough for a single colour");

// Group the per-orbit counts by graphlet-kind name for inspection.
let by_kind =
    counts.to_graphlet_names::<ExtendedGraphletType, u8>(graph.get_number_of_node_labels());

// The edge (0, 1) of a 4-clique lies in two triangles and one 4-clique.
assert_eq!(by_kind.get("Triangle"), Some(&2));
assert_eq!(by_kind.get("FourClique"), Some(&1));
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
