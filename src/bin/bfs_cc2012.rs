extern crate COST;

use std::fs::File;

use COST::graph_iterator::{EdgeMapper, DeltaCompressedReaderMapper, NodesEdgesMemMapper, UpperLowerMemMapper, ReaderMapper, CachingReaderMapper};
use std::io::BufReader;

pub fn print_output(labels: Vec<u32>) {
  for (i, x) in labels.into_iter().enumerate() {
    println!("{}\t{}", i, x);
  }
}

fn main() {

    if std::env::args().len() != 5 {
        println!("Usage: bfs (reader | hybrid | vertex | hilbert | compressed) <prefix> nodes");
        return;
    }

    let mode = std::env::args().nth(1).expect("mode unavailable");
    let name = std::env::args().nth(2).expect("name unavailable");
    let nodes: u32 = std::env::args().nth(3).expect("nodes unavailable").parse().expect("nodes not parseable");
    let start_vertex: u32 = std::env::args().nth(4).expect("start_vertex unavailable").parse().expect("start_vertex not parseable");

    let start = std::time::Instant::now();

    let mut label : Vec<u32> = Vec::new();

    match mode.as_str() {
        "reader" => {
            label = bfs::<false, _>(&ReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes, start_vertex);
        }
        "hybrid" => {
            let file = File::open(&name).unwrap();
            let len = file.metadata().unwrap().len();
            let ulen = (len >> 2) + 1;
            let llen = (len >> 1) + 1;
            label = bfs::<false, _>(&CachingReaderMapper::new(|| BufReader::new(File::open(&name).unwrap()), ulen as usize, llen as usize), nodes, start_vertex);
        }
        "vertex" => {
            label = bfs::<false, _>(&NodesEdgesMemMapper::new(&name), nodes, start_vertex);
        },
        "hilbert" => {
            label = bfs::<false, _>(&UpperLowerMemMapper::new(&name), nodes, start_vertex);
        },
        "compressed" => {
            label = bfs::<false, _>(&DeltaCompressedReaderMapper::new(|| BufReader::new(File::open(&name).unwrap())), nodes, start_vertex);
        },
        _ => { println!("unrecognized mode: {:?}", mode); },
    }

    let elapsed = start.elapsed();
    eprintln!("E2E runtime: {} ns", elapsed.as_nanos());
    print_output(label);
}

fn bfs<const OUT: bool, G: EdgeMapper>(graph: &G, nodes: u32, start_vertex: u32) -> Vec<u32> {

    let timer = std::time::Instant::now();

    let svert : usize = start_vertex as usize;

    let mut roots: Vec<u32> = (0..nodes).collect();

    let mut label : Vec<u32> = vec![std::u32::MAX; nodes as usize];

    label[svert] = 0;

    let mut num_edges: u64 = 0;

    graph.map_edges(|mut x, mut y| {

        if x == start_vertex { label[y as usize] = 1; }
        if y == start_vertex { label[x as usize] = 1; }

        x = unsafe { *roots.get_unchecked(x as usize) };
        y = unsafe { *roots.get_unchecked(y as usize) };

        unsafe { while x != *roots.get_unchecked(x as usize) { x = *roots.get_unchecked(x as usize); } }
        unsafe { while y != *roots.get_unchecked(y as usize) { y = *roots.get_unchecked(y as usize); } }

        // works for Hilbert curve order
        roots[x as usize] = ::std::cmp::min(x, y);
        roots[y as usize] = ::std::cmp::min(x, y);
        num_edges += 1;
    });

    for i in 0..nodes {
        let mut node = i;
        while node != roots[node as usize] { node = roots[node as usize]; }
        //if node != start_vertex { label[i as usize] = 0; }
    }

    let mut roots: Vec<(u32, u32)> = Vec::with_capacity(nodes as usize);

    for i in 0..nodes {
        if label[i as usize] == 1 { roots.push((i, start_vertex)); }
    }

    if OUT {
      eprintln!("{:?}\titeration: {}", timer.elapsed(), 0);
    }

    // WTF is this? What are YOU PLANNNING?!??!
    let mut edges = Vec::new();
    let mut iteration = 1;

    // iterate as long as there are changes
    while edges.len() == edges.capacity() {

        // allocate if the first iteration, clear otherwise
        if edges.capacity() == 0 { edges = Vec::with_capacity(num_edges as usize); }
        else                     { edges.clear(); }

        graph.map_edges(|src, dst| {

            let label_src = unsafe { *label.get_unchecked(src as usize) };
            let label_dst = unsafe { *label.get_unchecked(dst as usize) };

            if edges.len() < edges.capacity() {

                if (label_src > iteration && label_dst > iteration + 1) ||
                   (label_dst > iteration && label_src > iteration + 1) {
                    edges.push((src, dst));
                }
            }

            if label_src == iteration && label_dst > iteration + 1 {
                unsafe { *label.get_unchecked_mut(dst as usize) = iteration + 1; }
                roots.push((dst, src));
            }

            if label_dst == iteration && label_src > iteration + 1 {
                unsafe { *label.get_unchecked_mut(src as usize) = iteration + 1; }
                roots.push((src, dst));
            }
        });

        iteration += 1;
        if OUT {
            eprintln!("{:?}\titeration: {}", timer.elapsed(), iteration);
        }
    }

    let mut done = false;
    while !done {
        done = true;
        edges.retain(|&(src,dst)| {

            if label[src as usize] == iteration && label[dst as usize] > iteration + 1 {
                label[dst as usize] = iteration + 1;
                roots.push((dst, src));
                done = false;
            }
            else
            if label[dst as usize] == iteration && label[src as usize] > iteration + 1 {
                label[src as usize] = iteration + 1;
                roots.push((src, dst));
                done = false;
            }

            (label[src as usize] > iteration && label[dst as usize] > iteration + 1) ||
            (label[dst as usize] > iteration && label[src as usize] > iteration + 1)
        });

        iteration += 1;
        if OUT {
            eprintln!("{:?}\titeration: {}", timer.elapsed(), iteration);
        }
    }
    label
}
