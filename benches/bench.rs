#![feature(test)]

extern crate test;

use std::fs;

use bitbuffer::{BitReadBuffer, BitReadStream, LittleEndian};
use sourcenav::get_area_tree;
use std::fs::read;
use test::Bencher;

#[bench]
fn bench_badwater(b: &mut Bencher) {
    let file = read("data/pl_badwater.nav").unwrap();
    let data = BitReadStream::new(BitReadBuffer::new(file, LittleEndian));

    b.iter(|| {
        test::black_box(get_area_tree(data.clone()));
    })
}
