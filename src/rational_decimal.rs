use core::fmt;
use rayon::prelude::*;
use rug::{
    Integer,
    ops::{DivRoundingAssign, NegAssign, Pow},
};
use std::{ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Rem, Sub}, time::Instant};

#[derive(Debug, Clone)]
pub struct RationalDecimal {
    n: Integer,
    d: Integer,
}

#[allow(dead_code)]
impl RationalDecimal {
    fn reduce(&mut self) {
        let g = self.n.clone().gcd(&self.d);
        self.n.div_floor_assign(&g);
        self.d.div_floor_assign(&g);
    }

    pub fn new(numerator: Integer, denominator: Integer, reduce: bool) -> Self {
        let mut n = numerator;
        let mut d = denominator;

        if d == Integer::ZERO {
            panic!("Division by zero!!!");
        }

        if reduce {
            let g = n.clone().gcd(&d);
            n.div_floor_assign(&g);
            d.div_floor_assign(&g);
        }

        if d.is_negative() {
            n.neg_assign();
            d.neg_assign();
        }

        Self { n, d }
    }

    pub fn to_decimal_long_div(&self, precision: usize) -> String {
        let mut res: String = String::new();
        let integer_part = self.n.clone().div(&self.d);

        res.push_str(&integer_part.to_string());
        res.push('.');

        let mut remainder = self.n.clone().rem(&self.d);
        let mut position = 0;

        while remainder != Integer::ZERO && (precision == 0 || position < precision) {
            remainder.mul_assign(10);
            // let digit = &remainder / &self.d;
            let (digit, new_remainder) = remainder.div_rem(self.d.clone());
            res.push((digit.to_u8().unwrap() + b'0') as char);

            remainder = new_remainder;
            position += 1;
        }

        res
    }

    pub fn to_decimal_direct(&self, precision: usize) -> String {
        let scale = Integer::from(10u32).pow(precision as u32);

        // scale numerator
        let scaled = &self.n * scale;

        // single division
        let quotient = scaled / &self.d;

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

    pub fn zero() -> Self {
        Self {
            n: Integer::ZERO,
            d: Integer::from(1),
        }
    }

    pub fn one() -> Self {
        Self {
            n: Integer::from(1),
            d: Integer::from(1),
        }
    }
}

impl fmt::Display for RationalDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.d == Integer::from(1) {
            write!(f, "{}", self.n)
        } else {
            write!(f, "{}/{}", self.n, self.d)
        }
    }
}

impl From<i64> for RationalDecimal {
    fn from(value: i64) -> Self {
        RationalDecimal::new(Integer::from(value), Integer::from(1), true)
    }
}

impl<'a, 'b> Add<&'b RationalDecimal> for &'a RationalDecimal {
    type Output = RationalDecimal;

    fn add(self, other: &'b RationalDecimal) -> RationalDecimal {
        let numerator = Integer::from(&self.n * &other.d) + (&other.n * &self.d);
        let denominator = Integer::from(&self.d * &other.d);

        RationalDecimal::new(numerator, denominator, true)
    }
}

impl<'a> Add<i64> for &'a RationalDecimal {
    type Output = RationalDecimal;

    fn add(self, other: i64) -> RationalDecimal {
        self + &RationalDecimal::from(other)
    }
}

impl<'a> Add<&'a RationalDecimal> for i64 {
    type Output = RationalDecimal;

    fn add(self, other: &'a RationalDecimal) -> RationalDecimal {
        &RationalDecimal::from(self) + other
    }
}

impl<'a, 'b> Mul<&'b RationalDecimal> for &'a RationalDecimal {
    type Output = RationalDecimal;

    fn mul(self, other: &'b RationalDecimal) -> RationalDecimal {
        let numerator = Integer::from(&self.n * &other.n);
        let denominator = Integer::from(&self.d * &other.d);

        RationalDecimal::new(numerator, denominator, true)
    }
}

impl<'a> Mul<i64> for &'a RationalDecimal {
    type Output = RationalDecimal;

    fn mul(self, other: i64) -> RationalDecimal {
        self * &RationalDecimal::from(other)
    }
}

impl<'a> Mul<&'a RationalDecimal> for i64 {
    type Output = RationalDecimal;

    fn mul(self, other: &'a RationalDecimal) -> RationalDecimal {
        &RationalDecimal::from(self) * other
    }
}

impl<'a> Neg for &'a RationalDecimal {
    type Output = RationalDecimal;

    fn neg(self) -> RationalDecimal {
        RationalDecimal {
            n: Integer::from(-&self.n),
            d: self.d.clone(),
        }
    }
}

impl<'a, 'b> Sub<&'b RationalDecimal> for &'a RationalDecimal {
    type Output = RationalDecimal;

    fn sub(self, other: &'b RationalDecimal) -> RationalDecimal {
        self + &(-other)
    }
}

impl<'b> AddAssign<&'b RationalDecimal> for RationalDecimal {
    fn add_assign(&mut self, other: &'b RationalDecimal) {
        self.n = Integer::from(&self.n * &other.d) + Integer::from(&other.n * &self.d);
        self.d = Integer::from(&self.d * &other.d);
    }
}

#[allow(dead_code)]
fn rarctan_sequential(_x: i64, terms: i64) -> RationalDecimal {
    let x = Integer::from(_x);
    let mut result = RationalDecimal::zero();
    let x_sq = Integer::from(&x * &x);
    let mut power = Integer::from(x);

    for i in 0..terms {
        let sign = Integer::from(if i % 2 == 0 { 1 } else { -1 });
        let denom = Integer::from(&power * (2 * i + 1));
        let term = RationalDecimal::new(sign, denom, true);
        result += &term;

        power *= &x_sq;
    }

    result
}

#[allow(dead_code)]
fn rarctan_parallel(x: i64, terms: i64) -> RationalDecimal {
    (0..terms)
        .into_par_iter()
        .map(|i| {
            let sign = Integer::from(if i % 2 == 0 { 1 } else { -1 });

            let exp: u32 = 2 * i as u32 + 1;

            let power = Integer::from(x).pow(exp);
            let denom = Integer::from(&power * exp);

            RationalDecimal::new(sign, denom, true)
        })
        .reduce(|| RationalDecimal::zero(), |a, b| &a + &b)
}

#[allow(dead_code)]
fn rarctan_binary_split(_x: i64, terms: i64) -> RationalDecimal {
    let x = Integer::from(_x);
    let (p, q) = rarctan_binary_split_helper(0, terms, &x);

    let res = RationalDecimal::new(p, q, true);
    res
}

fn rarctan_binary_split_helper(a: i64, b: i64, q: &Integer) -> (Integer, Integer) {
    if b - a == 1 {
        let k = a;

        let mut denom = Integer::from(2 * k + 1);
        let q_pow = Integer::from(q.pow((2 * k + 1) as u32));

        denom *= q_pow;

        let mut num = Integer::from(1);

        if k % 2 == 1 {
            num = -num;
        }

        return (num, denom);
    }

    let m = (a + b) / 2;

    let (p1, q1) = rarctan_binary_split_helper(a, m, q);
    let (p2, q2) = rarctan_binary_split_helper(m, b, q);

    let p = Integer::from(&p1 * &q2) + &p2 * &q1;
    let q = q1 * q2;

    (p, q)
}

#[allow(dead_code)]
fn pi_sequential(arctan_terms: i64) -> String {
    let start = Instant::now();
    let results = vec![
        (6348, 2852),
        (1180, 4193),
        (2372, 4246),
        (1436, 39307),
        (1924, 55603),
        (2500, 211050),
        (-2832, 390112),
    ];

    let mut result = RationalDecimal::zero();

    for (c, a) in results.into_iter() {
        result += &(c * &rarctan_sequential(a, arctan_terms));
    }

    let pi_calc_time = start.elapsed();
    println!("Calculated, time taken: {}", pi_calc_time.as_secs_f64());

    result.to_decimal_long_div((arctan_terms * 7) as usize)
}

#[allow(dead_code)]
fn pi_parallel(arctan_terms: i64) -> String {
    let start = Instant::now();

    // let terms = [
    //     (6348, 2852),
    //     (1180, 4193),
    //     (2372, 4246),
    //     (1436, 39307),
    //     (1924, 55603),
    //     (2500, 211050),
    //     (-2832, 390112),
    // ];

    // let terms = [(16, 5), (-4, 239)];

    let terms = [
        (145848, 390112),
        (543632, 485298),
        (1098036, 683982),
        (-158324, 1984933),
        (713908, 2478328),
        (-458276, 3449051),
        (-586284, 18975991),
        (247656, 22709274),
        (-276176, 24208144),
        (-357724, 201229582),
        (-175752, 2189376182),
    ];

    let result = terms
        .par_iter()
        .map(|(c, a)| *c * &rarctan_binary_split(*a, arctan_terms))
        .reduce(|| RationalDecimal::zero(), |a, b| &a + &b);

    let pi_calc_time = start.elapsed();
    println!("Calculated, time taken: {}", pi_calc_time.as_secs_f64());

    result.to_decimal_direct((arctan_terms * 100) as usize)
}
