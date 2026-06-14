#![allow(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

mod test_from_csv;
use test_from_csv::test_from_csv;

#[test]
fn test_cora() {
    test_from_csv(
        "Cora",
        "tests/data/cora/node_list.csv",
        "tests/data/cora/edge_list.csv",
        [
            94855, 4890, 391_493, 195_709, 3_126_975, 6144, 53594, 53594, 107_188, 9872, 2468, 1320,
        ],
    );
}
