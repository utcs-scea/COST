extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper, ReaderMapper, CachingReaderMapper};
use std::io::BufReader;
use std::time::Instant;

fn main() {

    if std::env::args().len() != 3 {
        println!("Usage: stats  (reader | vertex | hilbert | compressed) <prefix>");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");

    let start = Instant::now();

    match mode.as_str() {
        "reader" => {
            stats(&ReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())));
        }
        "hybrid" => {
            let file = File::open(&name).unwrap();
            let len = file.metadata().unwrap().len();
            let ulen = (len >> 2) + 1;
            let llen = (len >> 1) + 1;
            stats(&CachingReaderMapper::new(|| BufReader::new(File::open(&name).unwrap()), ulen as usize, llen as usize));
        }
        "vertex" => {
            stats(&NodesEdgesMemMapper::new(&name));
        },
        "hilbert" => {
            stats(&UpperLowerMemMapper::new(&name));
        },
        "compressed" => {
            stats(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())));
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }

    let elapsed = start.elapsed();
    println!("E2E runtime: {} ns", elapsed.as_nanos());
}

fn stats<G: EdgeMapper>(graph: &G) -> u32 {
    let mut max_x = 0;
    let mut max_y = 0;
    let mut edges = 0u64;
    graph.map_edges(|x, y| {
        if max_x < x { max_x = x; }
        if max_y < y { max_y = y; }
        edges += 1;
    });

    println!("max x: {}", max_x);
    println!("max y: {}", max_y);
    println!("edges: {}", edges);
    std::cmp::max(max_x, max_y) + 1
}
