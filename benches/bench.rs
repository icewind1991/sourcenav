#![feature(test)]

extern crate test;

use std::fs;

use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
use sourcenav::{get_quad_tree, read_areas};
use std::fs::read;
use test::Bencher;

#[bench]
fn bench_badwater_areas(b: &mut Bencher) {
    let file = read("data/pl_badwater.nav").unwrap();
    let data = BitReadStream::new(BitReadBuffer::new(file, LittleEndian));

    b.iter(|| {
        test::black_box(read_areas(data.clone()));
    })
}

#[bench]
fn bench_badwater_quads(b: &mut Bencher) {
    let file = read("data/pl_badwater.nav").unwrap();
    let data = BitReadStream::new(BitReadBuffer::new(file, LittleEndian));

    b.iter(|| {
        test::black_box(get_quad_tree(data.clone()));
    })
}
