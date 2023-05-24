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
