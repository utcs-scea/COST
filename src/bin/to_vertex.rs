extern crate byteorder;
extern crate COST;

use std::io::{BufReader, BufWriter};
use std::fs::File;
use COST::graph_iterator::{EdgeMapper, ReaderMapper};
use byteorder::{WriteBytesExt, LittleEndian};

fn main() {

    if std::env::args().len() != 3 {
        println!("Usage: to_vertex <source> <prefix>");
        println!("NOTE: <prefix>.nodes and <prefix>.edges will be overwritten.");
        return;
    }

    let source = std::env::args().nth(1).expect("source unavailable"); let source = &source;
    let target = std::env::args().nth(2).expect("prefix unavailable"); let target = &target;

    let reader_mapper = ReaderMapper { reader: || BufReader::new(File::open(source).unwrap()) };

    let mut edge_writer = BufWriter::new(File::create(format!("{}.edges", target)).unwrap());
    let mut node_writer = BufWriter::new(File::create(format!("{}.nodes", target)).unwrap());

    let mut cnt = 0;
    let mut src = 0;

    let mut vec : Vec<Vec<u32>> = Vec::new();

    reader_mapper.map_edges(|x, y| {
        if x != src {
            if cnt > 0 {
                node_writer.write_u32::<LittleEndian>(src).ok().expect("write error");
                node_writer.write_u32::<LittleEndian>(cnt).ok().expect("write error");
                cnt = 0;
            }
            src = x;
        }

        edge_writer.write_u32::<LittleEndian>(y).ok().expect("write error");
        cnt += 1;

        let max = std::cmp::max(x, y) as usize;
        if max >= vec.len() {
            vec.resize(max + 1, vec![]);
        }
        if !vec[x as usize].contains(&y) {
            vec[x as usize].push(y);
        }
        if !vec[y as usize].contains(&x) {
            vec[y as usize].push(x);
        }
    });

    if cnt > 0 {
        node_writer.write_u32::<LittleEndian>(src).ok().expect("write error");
        node_writer.write_u32::<LittleEndian>(cnt).ok().expect("write error");
    }

    let mut edge_bi_writer = BufWriter::new(File::create(format!("{}.biedges", target)).unwrap());
    let mut node_bi_writer = BufWriter::new(File::create(format!("{}.binodes", target)).unwrap());

    for (i, edge_vec) in vec.iter().enumerate() {
        if edge_vec.is_empty() {
            continue;
        }
        node_bi_writer.write_u32::<LittleEndian>(i as u32).ok().expect("write error");
        node_bi_writer.write_u32::<LittleEndian>(edge_vec.len() as u32).ok().expect("write error");

        for edge in edge_vec {
            edge_bi_writer.write_u32::<LittleEndian>(*edge).ok().expect("write error");
        }
    }
}
