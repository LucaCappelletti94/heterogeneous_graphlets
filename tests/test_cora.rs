mod test_from_csv;
use test_from_csv::test_from_csv;


#[test]
fn test_cora() {
    test_from_csv(
        "Cora",
        "tests/data/cora/node_list.csv",
        "tests/data/cora/edge_list.csv",
    );
}
