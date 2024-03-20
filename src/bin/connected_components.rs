extern crate COST;
extern crate clap;

use std::fs::File;

use clap::Parser;
use std::io::BufReader;
use COST::const_switch_bool;
use COST::graph_iterator::{
    CachingReaderMapper, DeltaCompressedReaderMapper, EdgeMapper, Mapper, NodesEdgesMemMapper,
    ReaderMapper, UpperLowerMemMapper,
};

fn label_propagation<const OUT: bool, G: EdgeMapper>(graph: &G, nodes: u32) -> (u32, Vec<u32>) {
    let timer = std::time::Instant::now();
    let mut label: Vec<u32> = (0..nodes).collect();
    let mut old_sum: u64 = label.iter().fold(0, |t, x| t + *x as u64) + 1;
    let mut new_sum: u64 = old_sum - 1;
    let mut roots = nodes;
    if OUT {
        eprintln!("Metadata Set {:?}", timer.elapsed());
    }

    while new_sum < old_sum {
        old_sum = new_sum;
        graph.map_edges(
            |src, dst| match label[src as usize].cmp(&label[dst as usize]) {
                std::cmp::Ordering::Less => {
                    roots -= 1;
                    new_sum += label[src as usize] as u64;
                    new_sum -= label[dst as usize] as u64;
                    label[dst as usize] = label[src as usize];
                }
                std::cmp::Ordering::Greater => {
                    roots -= 1;
                    new_sum += label[dst as usize] as u64;
                    new_sum -= label[src as usize] as u64;
                    label[src as usize] = label[dst as usize];
                }
                std::cmp::Ordering::Equal => {}
            },
        );
        if OUT {
            eprintln!("Iteration {:?}", timer.elapsed());
        }
    }
    (roots, label)
}

#[derive(Parser, Debug)]
#[command(version, about = "Connected Components edge iterator application", long_about = None)]
struct Args {
    #[arg(short, long, action)]
    print_rounds: bool,

    #[arg(short, long)]
    mode: Mapper,

    #[arg(short, long)]
    filename: String,

    #[arg(short, long)]
    nodes: u32,
}

fn main() {
    let args = Args::parse();

    let mode = args.mode;
    let name = args.filename;
    let nodes: u32 = args.nodes;

    let start = std::time::Instant::now();

    let roots: u32;
    let label: Vec<u32>;

    match mode {
        Mapper::Reader => {
          (roots,label) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &ReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())),
                nodes,
            ));
        }
        Mapper::Hybrid => {
            let file = File::open(&name).unwrap();
            let len = file.metadata().unwrap().len();
            let ulen = (len >> 2) + 1;
            let llen = (len >> 1) + 1;
            (roots,label) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &CachingReaderMapper::new(
                    || BufReader::new(File::open(&name).unwrap()),
                    ulen as usize,
                    llen as usize,
                ),
                nodes,
            ));
        }
        Mapper::Vertex => {
          (roots, label) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &NodesEdgesMemMapper::new(&name),
                nodes
            ));
        }
        Mapper::Hilbert => {
          (roots, label) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &UpperLowerMemMapper::new(&name),
                nodes
            ));
        }
        Mapper::Compressed => {
          (roots, label) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())),
                nodes,
            ));
        }
    }

    let elapsed = start.elapsed();
    eprintln!("E2E runtime: {} ns", elapsed.as_nanos());
    println!("{} Connected Components", roots);
    for i in 0..label.len() {
      println!("{}\t{}", i, label[i]);
    }
}
