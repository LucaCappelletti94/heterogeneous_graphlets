mod test_from_csv;
use test_from_csv::test_from_csv;

#[test]
fn test_citeseer() {
    test_from_csv(
        "CiteSeer",
        "tests/data/citeseer/node_list.csv",
        "tests/data/citeseer/edge_list.csv",
    );
}
