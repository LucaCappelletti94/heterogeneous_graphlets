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
fn test_citeseer() {
    test_from_csv(
        "CiteSeer",
        "tests/data/citeseer/node_list.csv",
        "tests/data/citeseer/edge_list.csv",
        [
            46760, 3498, 222_306, 111_153, 667_890, 12376, 22900, 22900, 45800, 8800, 2200, 1530,
        ],
    );
}
