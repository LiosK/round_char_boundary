#![feature(test)]

extern crate test;

use std::{hint::black_box, sync::LazyLock};

use round_char_boundary::StrExt;

const LIMIT: usize = 20;

static TEXTS: LazyLock<Vec<String>> = LazyLock::new(|| {
    use rand::{SeedableRng as _, distr::Distribution as _, seq::IndexedRandom as _};
    use rand::{distr::Uniform, rngs::Xoshiro256PlusPlus};

    let mut charset = Vec::new();
    charset.extend(('A'..).take(32));
    charset.extend(('Α'..).take(32));
    charset.extend(('ぁ'..).take(32));
    charset.extend(('😀'..).take(32));
    let charlen = charset.iter().fold(0, |acc, c| acc + c.len_utf8()) / charset.len();

    let mut rng = Xoshiro256PlusPlus::seed_from_u64(16570);
    let len_range = Uniform::new(LIMIT - 8, LIMIT + 16).unwrap();

    let mut texts = Vec::new();
    for _ in 0..100 {
        let len = len_range.sample(&mut rng) / charlen;
        let iter = charset.choose_iter(&mut rng).unwrap();
        texts.push(iter.take(len).collect());
    }
    texts
});

#[bench]
fn floor_runtime_std(b: &mut test::Bencher) {
    let texts = &*TEXTS;
    b.iter(|| {
        for text in texts {
            black_box(text.floor_char_boundary(black_box(LIMIT)));
        }
    });
}

#[bench]
fn floor_runtime_unrolled(b: &mut test::Bencher) {
    let texts = &*TEXTS;
    b.iter(|| {
        for text in texts {
            black_box(text.floor_char_boundary_unrolled(black_box(LIMIT)));
        }
    });
}

#[bench]
fn floor_runtime_mask(b: &mut test::Bencher) {
    let texts = &*TEXTS;
    b.iter(|| {
        for text in texts {
            black_box(text.floor_char_boundary_mask(black_box(LIMIT)));
        }
    });
}

#[bench]
fn floor_const_std(b: &mut test::Bencher) {
    let texts = &*TEXTS;
    b.iter(|| {
        for text in texts {
            black_box(text.floor_char_boundary(LIMIT));
        }
    });
}

#[bench]
fn floor_const_unrolled(b: &mut test::Bencher) {
    let texts = &*TEXTS;
    b.iter(|| {
        for text in texts {
            black_box(text.floor_char_boundary_unrolled(LIMIT));
        }
    });
}

#[bench]
fn floor_const_mask(b: &mut test::Bencher) {
    let texts = &*TEXTS;
    b.iter(|| {
        for text in texts {
            black_box(text.floor_char_boundary_mask(LIMIT));
        }
    });
}
