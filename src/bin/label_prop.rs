extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper, ReaderMapper, CachingReaderMapper};
use std::io::BufReader;

fn main() {

    if std::env::args().len() != 4 {
        println!("Usage: label_propagation  (vertex | hilbert | compressed) <prefix> nodes");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args().nth(3).expect("nodes unavailable").parse().expect("nodes not parseable");

    let start = std::time::Instant::now();

    match mode.as_str() {
        "reader" => {
            label_propagation(&ReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes)
        }
        "hybrid" => {
            let file = File::open(&name).unwrap();
            let len = file.metadata().unwrap().len();
            let ulen = (len >> 2) + 1;
            let llen = (len >> 1) + 1;
            label_propagation(&CachingReaderMapper::new(|| BufReader::new(File::open(&name).unwrap()), ulen as usize, llen as usize), nodes);
        }
        "vertex" => {
            label_propagation(&NodesEdgesMemMapper::new(&name), nodes)
        },
        "hilbert" => {
            label_propagation(&UpperLowerMemMapper::new(&name), nodes)
        },
        "compressed" => {
            label_propagation(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes)
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }

    let elapsed = start.elapsed();
    println!("E2E runtime: {} ns", elapsed.as_nanos());
}

fn label_propagation<G: EdgeMapper>(graph: &G, nodes: u32) {
    let mut label: Vec<u32> = (0..nodes).collect();
    let mut old_sum: u64 = label.iter().fold(0, |t,x| t + *x as u64) + 1;
    let mut new_sum: u64 = label.iter().fold(0, |t,x| t + *x as u64);

    let mut edges = std::collections::HashSet::new();
    graph.map_edges(|src, dst| {
        if src != dst {
            let min = std::cmp::min(src, dst) as u64;
            let max = std::cmp::max(src, dst) as u64;
            edges.insert(min << 32 | max);
        }
    });
    // Double to make all edges bidirectional
    println!("{} edges", edges.len() * 2);

   let timer = std::time::Instant::now();

    while new_sum < old_sum {
        graph.map_edges(|src, dst| {
            match label[src as usize].cmp(&label[dst as usize]) {
                std::cmp::Ordering::Less    => label[dst as usize] = label[src as usize],
                std::cmp::Ordering::Greater => label[src as usize] = label[dst as usize],
                std::cmp::Ordering::Equal   => { },
            }
        });

        old_sum = new_sum;
        new_sum = label.iter().fold(0, |t,x| t + *x as u64);
        println!("Iteration {:?}", timer.elapsed());
    }

    let mut non_roots = 0u32;
    for i in 0..label.len() { if i as u32 != label[i] { non_roots += 1; }}
    println!("{} non-roots found in {:?}", non_roots, timer.elapsed());
}
