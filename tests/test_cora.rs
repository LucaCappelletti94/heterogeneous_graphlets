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
            92329, 7128, 391_781, 196_104, 3_129_981, 6144, 53594, 53594, 107_760, 9872, 2521, 1320,
        ],
    );
}
