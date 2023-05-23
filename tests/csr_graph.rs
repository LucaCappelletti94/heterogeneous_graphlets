use csv::Reader;

/// Compressed Sparse Row Graph
pub struct CSRGraph {
    /// The number of nodes in the graph.
    number_of_nodes: usize,
    /// The number of edges in the graph.
    number_of_edges: usize,
    /// The number of node labels in the graph.
    number_of_node_labels: usize,
    /// The node labels of the graph.
    node_labels: Vec<usize>,
    /// The offsets of the graph.
    offsets: Vec<usize>,
    /// The edges of the graph.
    edges: Vec<usize>,
}

fn read_csv(path: &str) -> Result<Vec<usize>, String> {
    let mut reader = Reader::from_path(path).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;
        let value = record[0].parse::<usize>().map_err(|e| e.to_string())?;
        result.push(value);
    }
    Ok(result)
}

impl CSRGraph {
    /// Create a new CSRGraph from the provided node list and edge list.
    ///
    /// # Arguments
    /// * `node_list_path` - The path to the node list.
    /// * `edge_list_path` - The path to the edge list.
    ///
    /// # Implementation details
    /// We expect the node list to be a csv file containing a single column
    /// with the NUMERIC node labels. The length of the column must be equal
    /// to the number of nodes in the graph. An example of the node list is
    /// the following:
    ///
    /// ```csv
    /// 0
    /// 1
    /// 2
    /// 1
    /// 4
    /// ```
    ///
    /// The edge list must be a csv file containing two columns with the NUMERIC
    /// source and destination node IDs. The length of the columns must be equal
    /// to the number of edges in the graph. An example of the edge list is the
    /// following:
    ///
    /// ```csv
    /// 0,1
    /// 1,2
    /// 2,3
    /// 1,4
    /// 4,5
    /// ```
    ///
    pub fn from_csv(node_list_path: &str, edge_list_path: &str) -> Result<Self, String> {
        let node_list = read_csv(node_list_path)?;
        let edge_list = read_csv(edge_list_path)?;

        let number_of_nodes = node_list.len();
        let number_of_edges = edge_list.len();

        let mut node_labels = Vec::with_capacity(number_of_nodes);
        let mut offsets = Vec::with_capacity(number_of_nodes + 1);
        let mut edges = Vec::with_capacity(number_of_edges);

        let mut current_offset = 0;
        let mut current_node = 0;
        let mut current_node_label = node_list[0];
        offsets.push(current_offset);

        for node_label in node_list {
            if node_label != current_node_label {
                current_node += 1;
                current_node_label = node_label;
                offsets.push(current_offset);
            }
            node_labels.push(current_node_label);
            current_offset += 1;
        }
        offsets.push(current_offset);

        for edge in edge_list {
            edges.push(edge);
        }

        Ok(Self {
            number_of_nodes,
            number_of_edges,
            number_of_node_labels: current_node + 1,
            node_labels,
            offsets,
            edges,
        })
    }
}
