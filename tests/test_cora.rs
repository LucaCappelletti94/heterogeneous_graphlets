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
            92329, 7128, 270_053, 196_850, 3_137_638, 5350, 45655, 35803, 109_673, 7802, 2521, 1320,
        ],
    );
}
