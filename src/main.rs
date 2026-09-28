use rayon::prelude::*;
use rug::{Integer, ops::Pow};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    time::Instant,
};
mod rational_decimal;

const A: i128 = 13591409;
const B: i128 = 545140134;
const C: i128 = 640320;
const C3_OVER_24: i128 = (C * C * C) / 24;

fn to_decimal_direct(numerator: &Integer, denominator: &Integer, precision: usize) -> String {
    // let scale = Integer::from(10u32).pow(precision as u32);
    let scale = Integer::from(1);

    // single division
    let quotient = (numerator * scale) / denominator;

    let mut s = quotient.to_string();

    if precision == 0 {
        return s;
    }

    // ensure enough digits for fractional part
    if s.len() <= precision {
        let zeros = precision + 1 - s.len();
        s = "0".repeat(zeros) + &s;
    }

    // let split = s.len() - precision;
    // // s.insert(split, '.');

    s
}

#[allow(dead_code)]
fn pi_chudnovsky(digits: i128) -> String {
    let extra = 10;
    let n = (digits / 14) + 1;
    println!("Chudnovsky terms: {n}");

    let block_size = n / 1023;
    println!("Block size: {block_size}");

    let blocks: Vec<_> = (0..n).step_by(block_size as usize).collect();

    blocks.par_iter().enumerate().for_each(|(idx, &start)| {
        let block_num = (start / block_size) + 1;
        println!("Starting block: {block_num}");

        let start_time = Instant::now();
        let end = (start + block_size).min(n);

        let mut p_local = Integer::new();
        let mut q_local = Integer::new();
        let mut t_local = Integer::new();

        pi_chudnovsky_binary_split(start, end, &mut p_local, &mut q_local, &mut t_local);
        write_block(&format!("blocks/block_{idx}"), &p_local, &q_local, &t_local);

        println!("Block {} took {:?}", block_num, start_time.elapsed());
    });

    println!("Starting collection");

    for i in 0..10 {
        let start = Instant::now();
        let mut files: Vec<_> = fs::read_dir("blocks")
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        files.sort_by_key(|file| {
            file.file_name()
                .to_string_lossy()
                .split('_')
                .nth(1)
                .unwrap()
                .split('.')
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap()
        });

        fn collect_blocks((idx, chunk): (usize, &[fs::DirEntry])) {
            // let (mut p_local, mut q_local, mut t_local) = (
            //     read_p(&chunk[0].path().to_str().unwrap()),
            //     read_q(&chunk[0].path().to_str().unwrap()),
            //     read_t(&chunk[0].path().to_str().unwrap()),
            // );
            // let q = read_q(&chunk[1].path().to_str().unwrap());
            // let old_p = p_local.clone();

            // p_local *= &read_p(&chunk[1].path().to_str().unwrap());
            // q_local *= &q;
            // t_local *= &q;
            // t_local += &old_p * &read_t(&chunk[1].path().to_str().unwrap());

            // write_block(
            //     &format!("blocks_tmp/block_{}", idx),
            //     &p_local,
            //     &q_local,
            //     &t_local,
            // );
            let b0p = &chunk[0].path().to_str().unwrap().to_string();
            let b0q = &chunk[0].path().to_str().unwrap().to_string();
            let b0t = &chunk[0].path().to_str().unwrap().to_string();
            let b1p = &chunk[1].path().to_str().unwrap().to_string();
            let b1q = &chunk[1].path().to_str().unwrap().to_string();
            let b1t = &chunk[1].path().to_str().unwrap().to_string();
            fs::create_dir_all(format!("blocks_tmp/block_{idx}")).unwrap();

            write_int(
                &format!("blocks_tmp/block_{idx}/p"),
                &(read_p(b0p) * read_p(b1p)),
            );

            write_int(
                &format!("blocks_tmp/block_{idx}/q"),
                &(read_q(b0q) * read_q(b1q)),
            );

            let mut t_local = read_t(b0t);

            t_local *= &read_q(b1q);
            t_local += &read_p(b0p) * read_t(b1t);

            write_int(&format!("blocks_tmp/block_{idx}/t"), &t_local)
        }

        files.chunks(2).enumerate().for_each(collect_blocks);

        fs::remove_dir_all("blocks").unwrap();
        fs::rename("blocks_tmp", "blocks").unwrap();
        fs::create_dir("blocks_tmp").unwrap();

        println!("Iteration: {i} took {:?}", start.elapsed());
    }

    let scale = Integer::from(10).pow((digits + extra) as u32);
    let scale_sq = scale.square();
    let sqrt_term = Integer::from(10005 * scale_sq).sqrt();

    let mut numerator = read_q("blocks/block_0");
    numerator *= sqrt_term;
    numerator *= 426880;
    benchmark(|| to_decimal_direct(&numerator, &read_t("blocks/block_0"), digits as usize))
}

const PARALLEL_THRESHOLD: i128 = 2i128.pow(10);

#[allow(dead_code)]
fn pi_chudnovsky_binary_split(a: i128, b: i128, p: &mut Integer, q: &mut Integer, t: &mut Integer) {
    if b - a == 1 {
        let (p_local, q_local) = if a == 0 {
            let p = Integer::from(1);
            let q = Integer::from(1);
            (p, q)
        } else {
            let mut p = Integer::from(6 * a - 5);
            p *= 2 * a - 1;
            p *= 6 * a - 1;
            let mut q = Integer::from(a);
            q *= a;
            q *= a;
            q *= C3_OVER_24;
            (p, q)
        };

        let mut t_local = Integer::from(&p_local * (A + B * a));
        if a % 2 == 1 {
            t_local = -t_local;
        }

        *p = p_local;
        *q = q_local;
        *t = t_local;

        return;
    }

    let m = (a + b) / 2;
    let mut p1 = Integer::new();
    let mut q1 = Integer::new();
    let mut t1 = Integer::new();
    let mut p2 = Integer::new();
    let mut q2 = Integer::new();
    let mut t2 = Integer::new();

    if b - a > PARALLEL_THRESHOLD {
        rayon::join(
            || pi_chudnovsky_binary_split(a, m, &mut p1, &mut q1, &mut t1),
            || pi_chudnovsky_binary_split(m, b, &mut p2, &mut q2, &mut t2),
        );
    } else {
        pi_chudnovsky_binary_split(a, m, &mut p1, &mut q1, &mut t1);
        pi_chudnovsky_binary_split(m, b, &mut p2, &mut q2, &mut t2);
    };

    *p = Integer::from(&p1 * &p2);
    *q = Integer::from(&q1 * &q2);
    *t = Integer::from(&q2 * &t1);
    *t += p1 * t2;
}

fn benchmark<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    println!("Started");
    let start = Instant::now();
    let result = f();
    println!("Time taken: {:?}", start.elapsed());
    return result;
}
use rug::integer::Order;

fn write_int(path: &str, x: &Integer) {
    let mut file = BufWriter::new(File::create(path).unwrap());

    let sign: i8 = if x.is_negative() { -1 } else { 1 };
    let bytes = x.to_digits::<u64>(Order::Lsf);

    file.write_all(&sign.to_le_bytes()).unwrap();
    file.write_all(&(bytes.len() as u64).to_le_bytes()).unwrap();

    // write u64 buffer directly as bytes
    let raw = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len() * 8) };
    file.write_all(raw).unwrap();
}

fn write_block(base: &str, p: &Integer, q: &Integer, t: &Integer) {
    fs::create_dir_all(base).unwrap();
    write_int(&format!("{base}/p"), p);
    write_int(&format!("{base}/q"), q);
    write_int(&format!("{base}/t"), t);
}

fn read_int(path: &str) -> Integer {
    let mut file = BufReader::new(File::open(path).unwrap());

    let mut sign_buf = [0u8; 1];
    file.read_exact(&mut sign_buf).unwrap();
    let sign = i8::from_le_bytes(sign_buf);

    let mut len_buf = [0u8; 8];
    file.read_exact(&mut len_buf).unwrap();
    let len = u64::from_le_bytes(len_buf) as usize;

    let mut raw = vec![0u8; len * 8];
    file.read_exact(&mut raw).unwrap();

    let limbs: Vec<u64> =
        unsafe { std::slice::from_raw_parts(raw.as_ptr() as *const u64, len).to_vec() };

    let mut x = Integer::from_digits(&limbs, Order::Lsf);
    if sign < 0 {
        x = -x;
    }

    x
}

fn read_p(base: &str) -> Integer {
    read_int(&format!("{base}/p"))
}

fn read_q(base: &str) -> Integer {
    read_int(&format!("{base}/q"))
}

fn read_t(base: &str) -> Integer {
    read_int(&format!("{base}/t"))
}
fn main() {
    println!("Rayon threads: {}", rayon::current_num_threads());
    let digits = 5_000_000_000;
    println!("digits={:?}", digits);

    fs::create_dir_all("blocks").unwrap();
    fs::create_dir_all("blocks_tmp").unwrap();

    let pi_calc = benchmark(|| pi_chudnovsky(digits));
    let mut precision = 0;

    let pi_real = fs::read_to_string("pi_1B.txt").unwrap();
    println!("Read file");

    for (calc, real) in pi_calc.chars().zip(pi_real.chars()) {
        if calc == real {
            precision += 1;
        } else {
            break;
        }
    }

    println!("Precision: {precision}");

    fs::write("pi_calc10B.txt", pi_calc).unwrap();

    fs::remove_dir_all("blocks").unwrap();
    fs::remove_dir_all("blocks_tmp").unwrap();

    // let (mut p_local, mut q_local, mut t_local) = (
    //     read_p("blocks/block_0"),
    //     read_q("blocks/block_0"),
    //     read_t("blocks/block_0"),
    // );
    // let old_p = p_local.clone();
    // let q = read_q("blocks/block_1");

    // p_local *= &read_p("blocks/block_1");
    // q_local *= &q;
    // t_local *= &q;
    // t_local += &old_p * &read_t("blocks/block_1");

    // write_block(&format!("blocks_tmp/block_0"), &p_local, &q_local, &t_local);
}
