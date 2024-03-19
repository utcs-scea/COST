extern crate COST;
extern crate clap;

use std::fs::File;

use clap::Parser;
use std::io::BufReader;
use COST::graph_iterator::{
    CachingReaderMapper, DeltaCompressedReaderMapper, EdgeMapper, NodesEdgesMemMapper,
    ReaderMapper, UpperLowerMemMapper,
};

#[derive(Parser, Debug)]
#[command(version, about = "PageRank edge iterator application", long_about = None)]
struct Args {
    #[arg(short, long, action)]
    print_rounds: bool,

    #[arg(short, long)]
    mode: String,

    #[arg(short, long)]
    filename: String,

    #[arg(short, long)]
    nodes: u32,
}

fn main() {
    if std::env::args().len() != 4 {
        println!(
            "Usage: pagerank  (reader | vertex | hybrid | hilbert | compressed) <prefix> nodes"
        );
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args()
        .nth(3)
        .expect("nodes unavailable")
        .parse()
        .expect("nodes not parseable");

    let start = std::time::Instant::now();
    match mode.as_str() {
        "reader" => {
            pagerank(
                &ReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())),
                nodes,
                0.85f32,
            );
        }
        "vertex" => pagerank(&NodesEdgesMemMapper::new(&name), nodes, 0.85f32),
        "hybrid" => {
            let file = File::open(&name).unwrap();
            let len = file.metadata().unwrap().len();
            let ulen = (len >> 2) + 1;
            let llen = (len >> 1) + 1;
            pagerank(
                &CachingReaderMapper::new(
                    || BufReader::new(File::open(&name).unwrap()),
                    ulen as usize,
                    llen as usize,
                ),
                nodes,
                0.85f32,
            );
        }
        "hilbert" => pagerank(&UpperLowerMemMapper::new(&name), nodes, 0.85f32),
        "compressed" => pagerank(
            &DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())),
            nodes,
            0.85f32,
        ),
        _ => {
            println!("unrecognized mode: {:?}", mode);
        }
    }
    let elapsed = start.elapsed();
    println!("E2E runtime: {} ns", elapsed.as_nanos());
}

fn pagerank<G: EdgeMapper>(graph: &G, nodes: u32, alpha: f32) {
    let timer = std::time::Instant::now();

    let mut src = vec![0f32; nodes as usize];
    let mut dst = vec![0f32; nodes as usize];
    let mut deg = vec![0f32; nodes as usize];

    graph.map_edges(|x, _| deg[x as usize] += 1f32);

    for _iteration in 0..20 {
        for node in 0..nodes {
            src[node as usize] = alpha * dst[node as usize] / deg[node as usize];
            dst[node as usize] = 1f32 - alpha;
        }

        // graph.map_edges(|x, y| { dst[y as usize] += src[x as usize]; });

        // UNSAFE:
        graph.map_edges(|x, y| unsafe {
            *dst.get_unchecked_mut(y as usize) += *src.get_unchecked(x as usize);
        });
        println!("Iteration {}:\t{:?}", _iteration, timer.elapsed());
    }

    let mut max_val = 0 as f32;
    for val in dst {
        if val > max_val {
            max_val = val;
        }
    }
    println!("Finished in {:?}, maxVal: {}", timer.elapsed(), max_val);
}
