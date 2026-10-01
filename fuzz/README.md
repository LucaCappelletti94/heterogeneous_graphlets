# Fuzzing

The graph targets check the crate's counts against the brute-force oracle behind the `oracle` feature, and `perfect_hash_roundtrip` checks that the graphlet kind survives the perfect hash. Run one from its seeds with:

```sh
mkdir -p fuzz/corpus/graphlet_counts
cargo +nightly fuzz run graphlet_counts fuzz/corpus/graphlet_counts fuzz/seeds/graphlet_counts
```

## Seed Corpus

Every target has a seed corpus in `fuzz/seeds/<target>/`, and the ClusterFuzzLite build fails for a target without one. The graph targets grow from the graph shapes drawn in `assets/graphlets` and `perfect_hash_roundtrip` from one input per graphlet kind, each reduced by libFuzzer's `-set_cover_merge=1` to the smallest set that keeps the same coverage.
