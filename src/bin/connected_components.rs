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

fn print_output(connected_components: u32, labels: Vec<u32>) {
    println!("{} Connected Components", connected_components);
    for (i, label) in labels.into_iter().enumerate() {
        println!("{}\t{}", i, label);
    }
}

fn label_propagation<const OUT: bool, G: EdgeMapper>(
    graph: &G,
    nodes: u32,
    timer: std::time::Instant,
) -> (u32, Vec<u32>) {
    let mut label: Vec<u32> = (0..nodes).collect();
    let mut new_sum: u64 = if nodes % 2 == 0 {
        (nodes as u64 >> 1) * (nodes as u64 - 1)
    } else {
        (nodes as u64) * ((nodes as u64 - 1) >> 1)
    };
    let mut old_sum: u64 = new_sum + 1;
    let mut roots = nodes;
    if OUT {
        eprintln!("Metadata Set {:?}", timer.elapsed());
    }

    while new_sum < old_sum {
        old_sum = new_sum;
        graph.map_edges(
            |src, dst| match label[src as usize].cmp(&label[dst as usize]) {
                std::cmp::Ordering::Less => {
                    if label[dst as usize] == dst {
                        roots -= 1;
                    }
                    new_sum += label[src as usize] as u64;
                    new_sum -= label[dst as usize] as u64;
                    label[dst as usize] = label[src as usize];
                }
                std::cmp::Ordering::Greater => {
                    if label[src as usize] == src {
                        roots -= 1;
                    }
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

    let ccs: u32;
    let labels: Vec<u32>;

    match mode {
        Mapper::Reader => {
            (ccs, labels) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &ReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())),
                nodes,
                start,
            ));
        }
        Mapper::Hybrid => {
            let file = File::open(&name).unwrap();
            let len = file.metadata().unwrap().len();
            let ulen = (len >> 2) + 1;
            let llen = (len >> 1) + 1;
            (ccs, labels) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &CachingReaderMapper::new(
                    || BufReader::new(File::open(&name).unwrap()),
                    ulen as usize,
                    llen as usize,
                ),
                nodes,
                start,
            ));
        }
        Mapper::Vertex => {
            (ccs, labels) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &NodesEdgesMemMapper::new(&name),
                nodes,
                start,
            ));
        }
        Mapper::Hilbert => {
            (ccs, labels) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &UpperLowerMemMapper::new(&name),
                nodes,
                start,
            ));
        }
        Mapper::Compressed => {
            (ccs, labels) = const_switch_bool!(args.print_rounds, |B| label_propagation::<B, _>(
                &DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())),
                nodes,
                start,
            ));
        }
    }

    let elapsed = start.elapsed();
    eprintln!("E2E runtime: {} ns", elapsed.as_nanos());
    print_output(ccs, labels);
}
