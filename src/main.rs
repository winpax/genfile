use std::{
    io::Write,
    mem::{transmute, MaybeUninit},
    path::PathBuf,
};

mod size;

use arrayvec::ArrayVec;
use clap::Parser;
use indicatif::ProgressBar;

#[derive(Debug, Parser)]
struct Args {
    #[clap(
        short,
        long,
        help = "The size of the output file. (Will be rounded to the nearest kilobyte)"
    )]
    size: size::Size,

    #[clap(
        short,
        long,
        help = "Whether to generate file containing random data. Otherwise it will be all zeros."
    )]
    random: bool,

    #[clap(short, long, help = "The output file path")]
    output: PathBuf,
}

// The size of each chunk in bytes
const CHUNK_SIZE: u64 = 4096;
// Larger chunks are faster to generate for random data, because we use a thread pool to generate them
#[cfg(unix)]
const RAND_CHUNK_SIZE: u64 = CHUNK_SIZE * 256;
#[cfg(windows)]
const RAND_CHUNK_SIZE: u64 = CHUNK_SIZE;

const BLANK_CHUNK: [u8; CHUNK_SIZE as usize] = [0; CHUNK_SIZE as usize];
// const BLANK_RAND_CHUNK: [u8; RAND_CHUNK_SIZE as usize] = [0; RAND_CHUNK_SIZE as usize];

fn rand_bytes(remainder: &mut ArrayVec<u8, { CHUNK_SIZE as usize }>, size: u64) {
    unsafe { remainder.set_len(size as usize) };

    remainder.iter_mut().for_each(|byte| {
        *byte = rand::random();
    });
}

unsafe fn rand_chunk() -> [u8; RAND_CHUNK_SIZE as usize] {
    #[allow(clippy::uninit_assumed_init)]
    let mut bytes = unsafe {
        MaybeUninit::<[MaybeUninit<u8>; RAND_CHUNK_SIZE as usize]>::uninit().assume_init()
    };

    for byte in &mut bytes {
        byte.write(rand::random());
    }

    unsafe { transmute(bytes) }
}

fn main() {
    let args = Args::parse();

    let size = args.size.to_bytes();

    let chunk_size = if args.random {
        RAND_CHUNK_SIZE
    } else {
        CHUNK_SIZE
    };

    let chunks = size / chunk_size;
    let remainder = size % chunk_size;

    let progress = ProgressBar::new(chunks + 1);

    let mut file = std::fs::File::create(args.output.clone()).unwrap();

    for _ in 0..chunks {
        if args.random {
            file.write_all(&unsafe { rand_chunk() })
        } else {
            file.write_all(&BLANK_CHUNK)
        }
        .unwrap();

        progress.inc(1);
    }

    if remainder > 0 {
        let mut remainder_vec =
            const { arrayvec::ArrayVec::<u8, { CHUNK_SIZE as usize }>::new_const() };

        if args.random {
            rand_bytes(&mut remainder_vec, remainder);
        } else {
            remainder_vec.extend(std::iter::repeat_n(0, remainder as usize));
        }

        assert_eq!(
            remainder_vec.len(),
            remainder as usize,
            "Remainder size does not match the expected size"
        );
        file.write_all(&remainder_vec).unwrap();

        progress.inc(1);
    }

    progress.finish();
}
