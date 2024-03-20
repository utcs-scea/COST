use crate::hilbert_curve::{convert_to_hilbert_and_execute, BytewiseCached};
use std::cell::Cell;
use std::io::Read;
use crate::typedrw::TypedMemoryMap;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Mapper {
    /// Read From normal File
    Reader,

    /// Read From normal File and Cache
    Hybrid,

    /// Read From vertex/edge file pair
    Vertex,

    /// Read from hilbert file pair
    Hilbert,

    /// Read from Delta Compressed file
    Compressed,
}

pub trait EdgeMapper {
    fn map_edges(&self, action: impl FnMut(u32, u32));
}

pub struct DeltaCompressedReaderMapper<R: Read, F: Fn() -> R> {
    reader: F,
}

impl<R: Read, F: Fn() -> R> DeltaCompressedReaderMapper<R, F> {
    pub fn new(reader: F) -> DeltaCompressedReaderMapper<R, F> {
        DeltaCompressedReaderMapper { reader: reader }
    }
}

impl<R: Read, F: Fn() -> R> EdgeMapper for DeltaCompressedReaderMapper<R, F> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut hilbert = BytewiseCached::new();
        let mut current = 0u64;
        let mut reader = (self.reader)();

        let mut delta = 0u64; // for accumulating a delta
        let mut depth = 0u8; // for counting number of zeros

        let mut buffer = vec![0u8; 1 << 16];
        while let Ok(read) = reader.read(&mut buffer[..]) {
            if read == 0 {
                // Reached EOF.
                break;
            }
            for &byte in &buffer[..read] {
                if byte == 0 && delta == 0 {
                    depth += 1;
                } else {
                    delta = (delta << 8) + (byte as u64);
                    if depth == 0 {
                        current += delta;
                        delta = 0;
                        let (x, y) = hilbert.detangle(current);
                        action(x, y);
                    } else {
                        depth -= 1;
                    }
                }
            }
        }
    }
}

pub struct DeltaCompressedSliceMapper<'a> {
    slice: &'a [u8],
}

impl<'a> DeltaCompressedSliceMapper<'a> {
    pub fn new(slice: &'a [u8]) -> DeltaCompressedSliceMapper<'a> {
        DeltaCompressedSliceMapper { slice: slice }
    }
}

impl<'a> EdgeMapper for DeltaCompressedSliceMapper<'a> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut hilbert = BytewiseCached::new();
        let mut current = 0u64;

        let mut cursor = 0;
        while cursor < self.slice.len() {
            let byte = unsafe { *self.slice.get_unchecked(cursor) };
            cursor += 1;

            if byte > 0 {
                current += byte as u64;
                let (x, y) = hilbert.detangle(current);
                action(x, y);
            } else {
                let mut depth = 2;
                while unsafe { *self.slice.get_unchecked(cursor) } == 0 {
                    cursor += 1;
                    depth += 1;
                }
                let mut delta = 0u64;
                while depth > 0 {
                    delta = (delta << 8) + (unsafe { *self.slice.get_unchecked(cursor) } as u64);
                    cursor += 1;
                    depth -= 1;
                }

                current += delta;
                let (x, y) = hilbert.detangle(current);
                action(x, y);
            }
        }
    }
}

pub struct UpperLowerMemMapper {
    upper: TypedMemoryMap<((u16, u16), u32)>,
    lower: TypedMemoryMap<(u16, u16)>,
}

impl UpperLowerMemMapper {
    pub fn new(graph_name: &str) -> UpperLowerMemMapper {
        UpperLowerMemMapper {
            upper: TypedMemoryMap::new(format!("{}.upper", graph_name)),
            lower: TypedMemoryMap::new(format!("{}.lower", graph_name)),
        }
    }
}

impl EdgeMapper for UpperLowerMemMapper {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut slice = &self.lower[..];
        for &((u16_x, u16_y), count) in &self.upper[..] {
            let u16_x = (u16_x as u32) << 16;
            let u16_y = (u16_y as u32) << 16;
            for &(l16_x, l16_y) in &slice[..count as usize] {
                action(u16_x | l16_x as u32, u16_y | l16_y as u32);
            }

            slice = &slice[count as usize..];
        }
    }
}

pub struct NodesEdgesMemMapper {
    nodes: TypedMemoryMap<(u32, u32)>,
    edges: TypedMemoryMap<u32>,
}

impl NodesEdgesMemMapper {
    pub fn new(graph_name: &str) -> NodesEdgesMemMapper {
        NodesEdgesMemMapper {
            nodes: TypedMemoryMap::new(format!("{}.nodes", graph_name)),
            edges: TypedMemoryMap::new(format!("{}.edges", graph_name)),
        }
    }
}

impl EdgeMapper for NodesEdgesMemMapper {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut slice = &self.edges[..];
        for &(node, count) in &self.nodes[..] {
            for &edge in &slice[..count as usize] {
                action(node, edge);
            }

            slice = &slice[count as usize..];
        }
    }
}

pub struct ReaderMapper<B: ::std::io::BufRead, F: Fn() -> B> {
    pub reader: F,
}

impl<B: ::std::io::BufRead, F: Fn() -> B> ReaderMapper<B, F> {
    pub fn new(reader: F) -> ReaderMapper<B, F> {
        ReaderMapper { reader: reader }
    }
}

impl<R: ::std::io::BufRead, RF: Fn() -> R> EdgeMapper for ReaderMapper<R, RF> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let reader = (self.reader)();
        for readline in reader.lines() {
            let line = readline.ok().expect("read error");
            if !line.starts_with('#') {
                let mut elts = line[..].split_whitespace();
                let src: u32 = elts.next().unwrap().parse().ok().expect("malformed src");
                let dst: u32 = elts.next().unwrap().parse().ok().expect("malformed dst");
                action(src, dst);
            }
        }
    }
}

pub struct CachingReaderMapper<B: ::std::io::BufRead, F: Fn() -> B> {
    reader: ReaderMapper<B, F>,
    upper: Cell<Vec<((u16, u16), u32)>>,
    lower: Cell<Vec<(u16, u16)>>,
    is_cached: Cell<bool>,
}

impl<B: ::std::io::BufRead, F: Fn() -> B> CachingReaderMapper<B, F> {
    pub fn new(reader: F, cap_upper: usize, cap_lower: usize) -> CachingReaderMapper<B, F> {
        let mut upper = Vec::new();
        upper.reserve(cap_upper);
        let mut lower = Vec::new();
        lower.reserve(cap_lower);
        CachingReaderMapper::<B, F> {
            reader: ReaderMapper::new(reader),
            upper: Cell::new(upper),
            lower: Cell::new(lower),
            is_cached: Cell::new(false),
        }
    }
}

impl<B: ::std::io::BufRead, F: Fn() -> B> EdgeMapper for CachingReaderMapper<B, F> {
    fn map_edges(&self, mut action: impl FnMut(u32, u32)) {
        let mut upper = self.upper.take();
        let mut lower = self.lower.take();
        if !self.is_cached.get() {
            convert_to_hilbert_and_execute(&self.reader, false, action, |ux, uy, c, ls| {
                upper.push(((ux, uy), c));
                for &(lx, ly) in ls.iter() {
                    lower.push((lx, ly));
                }
            });
            self.is_cached.set(true);
        } else {
            let mut slice = &lower[..];
            for &((u16_x, u16_y), count) in upper.iter() {
                let u16_x = (u16_x as u32) << 16;
                let u16_y = (u16_y as u32) << 16;
                for &(l16_x, l16_y) in &slice[0..count as usize] {
                    action(u16_x | l16_x as u32, u16_y | l16_y as u32);
                }
                slice = &slice[count as usize..];
            }
        }
        self.upper.set(upper);
        self.lower.set(lower);
    }
}
