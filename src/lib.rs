#![feature(float_extras)]
extern crate geo_types;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
pub use geo_types::Coordinate;
// use core::num::dec2flt::rawfp::RawFloat;

fn debug_float(f: f64) -> () {
    // let (mantissa, exponent, sign) = f.integer_decode();
    println!("{}", f);
    let long: u64 = unsafe { std::mem::transmute(f) };
    // println!("{:b}{:b}{:b}", sign, exponent, mantissa);
    println!("{:#066b}", long);

    // let longMultiplied = (f * 0x80000000l) & 0x7fffffffl;
    // let longBits: u64 = unsafe { std::mem::transmute(longMultiplied) };
    // println!("{:#066b}", longBits);
}

fn widen(mut low_32: u64) -> u64 {
  low_32 |= low_32 << 16; low_32 &= 0x0000ffff0000ffff;
  low_32 |= low_32 << 8;  low_32 &= 0x00ff00ff00ff00ff;
  low_32 |= low_32 << 4;  low_32 &= 0x0f0f0f0f0f0f0f0f;
  low_32 |= low_32 << 2;  low_32 &= 0x3333333333333333;
  low_32 |= low_32 << 1;  low_32 &= 0x5555555555555555;
  low_32
}


const mult: f64 = -(0x80000000 as f64);

pub fn encode(point: Coordinate<f64>, bits: usize) -> u64 {
    let biased_lon = (point.x + 180.0) / 360.0;
    let biased_lat = (point.y + 90.0) / 180.0;
    debug_float(biased_lon);
    debug_float(biased_lat);

    let lat_bits: u64 = ((biased_lat * mult) as u64) & 0x7fffffff;
    let lon_bits: u64 = ((biased_lon * mult) as u64) & 0x7fffffff;

    // println!("{}", mult);
    println!("{}", lon_bits);
    println!("{}", lat_bits);
    println!("{:#066b}", lon_bits);
    println!("{:#066b}", lat_bits);

    let lat_wide = widen(lat_bits);
    let lon_wide = widen(lon_bits);
    println!("{:#066b}", lon_wide);
    println!("{:#066b}", lat_wide);

    let encoded = lon_wide | (lat_wide >> 1);
    println!("****************8");
    println!("{:#066b}", encoded);
    encoded >> (61 - bits)
}

static BASE32: &'static [char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'b',
                                   'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p',
                                   'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

lazy_static! {
    static ref BASE32_INV_FAST: [u8; 123] = {
        let mut base_32: [u8; 123] = [0; 123];
        for (idx, c) in BASE32.iter().enumerate() {
            let i: u8 = idx as u8;
            base_32[*c as usize] = i as u8;
        }
        base_32
    };

    static ref BASE32_INV: HashMap<usize, char> = {
        let mut m = HashMap::new();
        for (i, c) in BASE32.iter().enumerate() {
            m.insert(i, c.clone());
        }
        m
    };
}

use std::collections::VecDeque;

pub fn to_base_32(mut gh: u64, bits: usize) -> String {
    let mut chars: VecDeque<char> = VecDeque::with_capacity(bits / 5);;
    for i in (0..bits / 5).rev() {
        let lookup_index = (gh & 0x1f) as usize;
        println!("{}", lookup_index);
        let letter = BASE32[lookup_index];
        chars.push_front(letter);
        // chars.push_front(BASE32_INV[&((gh & 0x1f) as usize)]);
        gh >>= 5;
    }
    chars.into_iter().collect()
}

pub fn encode_base_32(point: Coordinate<f64>, bits: usize) -> String {
    for (index, char_code) in BASE32_INV_FAST.iter().enumerate() {
        println!("{} - {}", index, char_code);
    }
    to_base_32(encode(point, bits), bits)
}

#[cfg(test)]
mod tests {
    use geo_types::Coordinate;
    use {encode, encode_base_32};
    #[test]
    fn encoding() {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        assert_eq!(encode(point, 60), 1040636137860004224);
        assert_eq!("ww8p1r4t8", encode_base_32(point, 45));
    }
}
