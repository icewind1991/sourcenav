#![feature(test)]

extern crate test;

use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
use sourcenav::{get_quad_tree, read_areas};
use std::fs::read;
use test::Bencher;

#[bench]
fn bench_badwater_areas(b: &mut Bencher) {
    let file = read("data/pl_badwater.nav").unwrap();
    let data = BitReadStream::new(BitReadBuffer::new(file, LittleEndian));

    b.iter(|| {
        let _ = test::black_box(read_areas(data.clone()));
    })
}

#[bench]
fn bench_badwater_quads(b: &mut Bencher) {
    let file = read("data/pl_badwater.nav").unwrap();
    let data = BitReadStream::new(BitReadBuffer::new(file, LittleEndian));

    b.iter(|| {
        let _ = test::black_box(get_quad_tree(data.clone()));
    })
}

#[bench]
fn bench_tree_query(b: &mut Bencher) {
    let file = read("data/pl_badwater.nav").unwrap();
    let tree = get_quad_tree(file).unwrap();

    b.iter(|| {
        test::black_box(tree.find_best_height(320.0, -1030.0, 0.0));
    })
}
