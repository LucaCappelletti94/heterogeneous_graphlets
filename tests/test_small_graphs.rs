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
fn test_four_path() {
    test_from_csv(
        "Four path",
        "tests/data/four_path/node_list.csv",
        "tests/data/four_path/edge_list.csv",
    );
}

#[test]
fn test_four_star() {
    test_from_csv(
        "Four star",
        "tests/data/four_star/node_list.csv",
        "tests/data/four_star/edge_list.csv",
    );
}
