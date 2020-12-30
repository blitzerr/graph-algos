use std::{fs::File, io::Write, path::Path, vec};

use petgraph::dot::{Config, Dot};
use petgraph::{Directed, Graph as PG};

type VertexId = usize;
type Weight = i32;

/// All edges are directed.
#[derive(Debug)]
struct Edge {
    /// The weight of the edge. The weight is 0 for un-weighted edges. Weights can be negative.
    wt: Weight,

    /// All edges are directed and they have a "from" vertex and a "to" vertex. An edge is owned by
    /// a vertex and the owning vertex is the implicit vertex. The "id" field stores the id of the
    /// other (non-implicit vertex), which happens to be the id of the "to" vertex for the `out`
    /// edges and the ids of the "from" vertex for the "in_edges".
    other_id: VertexId,
}

#[derive(Debug)]
struct Vertex<T> {
    /// The the data item the node stores.
    data: T,
    /// The id of this node. A reference to the node is not used rather
    /// its the node id that is used as a reference to the node.
    id: VertexId,

    /// The list of edges eminating out of this node.
    out_edges: Vec<Edge>,

    /// The list of edges coming into this node.
    in_edges: Vec<Edge>,
}

impl<T> Vertex<T> {
    fn new(id: VertexId, data: T) -> Self {
        Self {
            data,
            id,
            out_edges: vec![],
            in_edges: vec![],
        }
    }

    fn add_out(&mut self, id: VertexId, wt: Weight) {
        self.out_edges.push(Edge { wt, other_id: id });
    }

    fn add_in(&mut self, id: VertexId, wt: Weight) {
        self.in_edges.push(Edge { wt, other_id: id });
    }
}

pub struct Graph<T> {
    /// This is the pool of all the vertices. A vertex is referenced by the index in this vector.
    /// As new nodes are added they get added here. Whn nodes are deleted, then they are removed
    /// from here. When nodes are removed, it leaves an unused index which should be vacuumed
    /// away.
    vertices: Vec<Vertex<T>>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Self { vertices: vec![] }
    }

    /// A create node creates a node with the provided data and returns the id of the node. This id
    /// should be used to retrieve the node.
    fn create_node(&mut self, data: T) -> VertexId {
        let next_idx = self.vertices.len() as VertexId;
        let vx = Vertex::new(next_idx, data);
        self.vertices.push(vx);
        next_idx
    }

    fn get_mut_data(&mut self, id: VertexId) -> Option<&mut T> {
        self.vertices.get_mut(id as usize).map(|v| &mut v.data)
    }

    fn add_weighted_edge(&mut self, from_id: VertexId, to_id: VertexId, weight: Weight) {
        self.vertices
            .get_mut(from_id as usize)
            .map(|vx| vx.add_out(to_id, weight));
    }

    fn add_edge(&mut self, from_id: VertexId, to_id: VertexId) {
        self.add_weighted_edge(from_id, to_id, 0)
    }

    fn draw(&self, filename: &str) {
        let edges = self
            .vertices
            .iter()
            .flat_map(|vx| {
                vx.out_edges
                    .iter()
                    .map(|out| (vx.id, out.other_id))
                    .collect::<Vec<(VertexId, VertexId)>>()
            })
            .collect::<Vec<(VertexId, VertexId)>>();
        println!("{:?}", edges);

        let mut graph =
            PG::<_, Weight, Directed, VertexId>::with_capacity(self.vertices.len(), edges.len());
        self.vertices.iter().for_each(|vx| {
            graph.add_node(vx.id);
        });
        graph.extend_with_edges(&edges);
        write_to_file(
            format!("{}.dot", filename),
            format!("{}", Dot::with_config(&graph, &[Config::EdgeNoLabel])),
        );

        println!(
            "Run: dot -Tpng {0}.dot -o {0}.png \nRun: open -a Preview {0}.png",
            filename
        );
    }
}

fn write_to_file(filename: String, data: String) {
    let path = Path::new(&filename);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

#[cfg(test)]
mod tests {
    use crate::Graph;

    #[test]
    fn it_works() {
        let mut g = Graph::new();
        let a = g.create_node(0);
        let b = g.create_node(1);
        let c = g.create_node(2);

        g.add_weighted_edge(a, c, 1);
        g.add_weighted_edge(b, c, 2);

        assert_eq!(g.get_mut_data(a).unwrap(), &mut 0);
        assert_eq!(g.get_mut_data(b).unwrap(), &mut 1);
        assert_eq!(g.get_mut_data(c).unwrap(), &mut 2);

        g.draw("my_graph");
    }
}
