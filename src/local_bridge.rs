use std::collections::HashMap;

use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::visit::{depth_first_search, DfsEvent};

pub struct LocalBridgeFinder {
    entry_times: HashMap<NodeIndex<u32>, usize>,
    tree_edges: HashMap<NodeIndex<u32>, Vec<NodeIndex<u32>>>,
    back_edges: HashMap<NodeIndex<u32>, Vec<NodeIndex<u32>>>,
}

/// Encapsulates the algorithm described here:
///
///   https://cp-algorithms.com/graph/bridge-searching.html
impl LocalBridgeFinder {
    pub fn new<N, E>(g: &UnGraph<N, E>, start: NodeIndex<u32>) -> Self {
        let mut entry_times: HashMap<NodeIndex<u32>, usize> = HashMap::new();
        let mut tree_edges: HashMap<NodeIndex<u32>, Vec<NodeIndex<u32>>> = HashMap::new();
        let mut back_edges: HashMap<NodeIndex<u32>, Vec<NodeIndex<u32>>> = HashMap::new();

        depth_first_search(g, Some(start), |event| match event {
            DfsEvent::Discover(n, time) => {
                // println!("EVENT Discover({}) = {}", n.index(), time.0);
                entry_times.insert(n, time.0);
            }
            DfsEvent::TreeEdge(n1, n2) => {
                // println!("EVENT TreeEdge({}, {})", n1.index(), n2.index());
                let entry = tree_edges.entry(n1).or_insert_with(|| vec![]);
                entry.push(n2);
            }
            DfsEvent::BackEdge(n1, n2) => {
                // println!("EVENT BackEdge({}, {})", n1.index(), n2.index());
                let entry = back_edges.entry(n1).or_insert_with(|| vec![]);
                entry.push(n2);
            }
            _ => {}
        });

        LocalBridgeFinder {
            entry_times,
            tree_edges,
            back_edges,
        }
    }

    fn lowest_entry_time(&self, n: NodeIndex<u32>, from: NodeIndex<u32>) -> Option<usize> {
        let entry_time = *self.entry_times.get(&n)?;
        let mut times = vec![entry_time];

        if let Some(prev_nodes) = self.back_edges.get(&n) {
            // println!("back edges for {}: {:?}", n.index(), prev_nodes);
            for prev_node in prev_nodes {
                if prev_node != &from {
                    times.push(*self.entry_times.get(prev_node)?);
                }
            }
        }

        if let Some(to_nodes) = self.tree_edges.get(&n) {
            // println!("tree edges for {}: {:?}", n.index(), to_nodes);
            for to_node in to_nodes {
                times.push(self.lowest_entry_time(*to_node, from)?);
            }
        }

        let min = times.iter().min().map(|u| *u);

        // println!("low({}) = {:?} // min({:?})", n.index(), min, times);

        min
    }

    pub fn is_local_bridge(&self, from: NodeIndex<u32>, to: NodeIndex<u32>) -> Option<bool> {
        let from_entry_time = *self.entry_times.get(&from)?;
        let to_lowest_entry_time = self.lowest_entry_time(to, from)?;

        // println!("  {} > {}", to_lowest_entry_time, from_entry_time);
        Some(to_lowest_entry_time > from_entry_time)
    }
}

#[test]
fn test_it_works() {
    let g = UnGraph::<u32, ()>::from_edges(&[
        // Clique A
        (1, 2),
        (2, 3),
        (3, 1),
        // Bridge
        (1, 4),
        // Clique B
        (4, 5),
        (5, 6),
        (6, 4),
    ]);

    let lbf = LocalBridgeFinder::new(&g, 2.into());

    assert_eq!(lbf.is_local_bridge(1.into(), 101.into()), None);
    assert_eq!(lbf.is_local_bridge(100.into(), 1.into()), None);
    assert_eq!(lbf.is_local_bridge(100.into(), 101.into()), None);
    assert_eq!(lbf.is_local_bridge(1.into(), 2.into()), Some(false));
    assert_eq!(lbf.is_local_bridge(1.into(), 4.into()), Some(true));
}
