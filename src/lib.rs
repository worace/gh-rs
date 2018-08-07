extern crate geo_types;
#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
pub use geo_types::Coordinate;

// fn debug_float(f: f64) -> () {
//     println!("{}", f);
//     let long: u64 = unsafe { std::mem::transmute(f) };
//     println!("{:#066b}", long);
// }

fn widen(mut low_32: u64) -> u64 {
  low_32 |= low_32 << 16; low_32 &= 0x0000ffff0000ffff;
  low_32 |= low_32 << 8;  low_32 &= 0x00ff00ff00ff00ff;
  low_32 |= low_32 << 4;  low_32 &= 0x0f0f0f0f0f0f0f0f;
  low_32 |= low_32 << 2;  low_32 &= 0x3333333333333333;
  low_32 |= low_32 << 1;  low_32 &= 0x5555555555555555;
  low_32
}

const FLOAT_SHIFT: f64 = 0x80000000u32 as f64;

pub fn encode(point: Coordinate<f64>, bits: usize) -> u64 {
    let biased_lon = (point.x + 180.0) / 360.0;
    let biased_lat = (point.y + 90.0) / 180.0;

    let lat_bits: u64 = ((biased_lat * FLOAT_SHIFT) as u64) & 0x7fffffff;
    let lon_bits: u64 = ((biased_lon * FLOAT_SHIFT) as u64) & 0x7fffffff;

    let lat_wide = widen(lat_bits);
    let lon_wide = widen(lon_bits);

    let encoded = lon_wide | (lat_wide >> 1);
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
    for _ in (0..bits / 5).rev() {
        let lookup_index = (gh & 0x1f) as usize;
        let letter = BASE32[lookup_index];
        chars.push_front(letter);
        gh >>= 5;
    }
    chars.into_iter().collect()
}

pub fn encode_base_32(point: Coordinate<f64>, bits: usize) -> String {
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
