#![feature(test)]
extern crate geo_types;
#[macro_use]
extern crate lazy_static;

use std::collections::VecDeque;
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

/**
 * "Unwidens" each bit by removing the zero from its left. This is the
 * inverse of "widen", but does not assume that the widened bits are padded
 * with zero.
 *
 * http://fgiesen.wordpress.com/2009/12/13/decoding-morton-codes/
 */
fn unwiden(mut wide: u64) -> u64 {
    wide &= 0x5555555555555555;
    wide ^= wide >> 1;
    wide &= 0x3333333333333333;
    wide ^= wide >> 2;
    wide &= 0x0f0f0f0f0f0f0f0f;
    wide ^= wide >> 4;
    wide &= 0x00ff00ff00ff00ff;
    wide ^= wide >> 8;
    wide &= 0x0000ffff0000ffff;
    wide ^= wide >> 16;
    wide &= 0x00000000ffffffff;
    wide
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

pub fn to_base_32(mut gh: u64, bits: usize) -> String {
    // let mut chars: VecDeque<char> = VecDeque::with_capacity(bits / 5);;
    let mut chars: [u8; 12] = [0; 12];
    for i in (0..bits / 5).rev() {
        let lookup_index = (gh & 0x1f) as usize;
        let letter = BASE32[lookup_index];
        chars[i] = letter as u8;
        gh >>= 5;
    }
    // chars.into_iter().collect()
    // String::from_utf8_lossy(&chars[0..bits / 5]).into_owned()
    let mut vec_of_u8 = vec![];
    vec_of_u8.extend_from_slice(&chars);
    String::from_utf8(vec_of_u8).unwrap()
}

pub fn encode_base_32(point: Coordinate<f64>, bits: usize) -> String {
    to_base_32(encode(point, bits), bits)
}

pub fn decode_bbox(gh: u64, bits: usize) -> (Coordinate<f64>, Coordinate<f64>) {
    let shifted = gh << (61 - bits);

    let lat = (((unwiden(shifted >> 1) & 0x3fffffff) as f64 / 0x40000000 as f64) * 180.0) - 90.0;
    let lon = (((unwiden(shifted) & 0x7fffffff) as f64 / 0x80000000u32 as f64) * 360.0) - 180.0;

    let mut error = 1.0;
    if (bits & 32) != 0 {
        error *= 0.25;
    }
    if (bits & 16) != 0 {
        error *= 0.5;
        error *= error;
    }
    if (bits & 8) != 0 {
        error *= 0.5;
        error *= error;
    }
    if (bits & 4) != 0 {
        error *= 0.5;
        error *= error;
    }
    if (bits & 2) != 0 {
        error *= 0.5;
    }

	  let lat_error = error * 90.0;
    let lon_error = if (bits & 1) == 0 {
        error * 180.0
    } else {
        error * 90.0
    };
    (Coordinate { x: lon - lon_error, y: lat - lat_error},
     Coordinate { x: lon + lon_error, y: lat + lat_error})
}

#[cfg(test)]
mod tests {
    extern crate test;
    extern crate geohash;

    use geo_types::Coordinate;
    use {encode, encode_base_32, decode_bbox};
    use self::test::Bencher;

    fn assert_coord(l: Coordinate<f64>, r: Coordinate<f64>) {
        let delta = 0.00001;
        let x_diff = (l.x - r.x).abs();
        let y_diff = (l.y - r.y).abs();
        assert!(x_diff < delta, format!("{} differed from {} by more than {}", l.x, r.x, delta));
        assert!(y_diff < delta, format!("{} differed from {} by more than {}", l.x, r.x, delta));
    }

    #[test]
    fn encoding() {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        assert_eq!(encode(point, 60), 1040636137860004224);
        assert_eq!("ww8p1r4t8", encode_base_32(point, 45));
        let bottom_left = Coordinate {
            x: 112.55839973688126,
            y: 37.832399904727936
        };
        let top_right = Coordinate {
            x: 112.55840007215738,
            y: 37.832400072366
        };

        let decoded = decode_bbox(1040636137860004224, 60);

        assert_coord(bottom_left, decoded.0);
        assert_coord(top_right, decoded.1);
    }

    #[bench]
    fn bench_gh_rs_encode(b: &mut Bencher) {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        b.iter(|| {
            for _i in 1..1000 {
                test::black_box(encode(point, 60));
            }
        })
    }

    #[bench]
    fn bench_geohash_encode(b: &mut Bencher) {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        b.iter(|| {
            // let n = test::black_box(1000000);
            for _i in 1..1000 {
                test::black_box(geohash::encode_long(point, 60));
            }
        })
    }

    #[bench]
    fn bench_gh_rs_encode_string(b: &mut Bencher) {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        b.iter(|| {
            for _i in 1..1000 {
                test::black_box(encode_base_32(point, 60));
            }
        })
    }

    #[bench]
    fn bench_geohash_encode_string(b: &mut Bencher) {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        b.iter(|| {
            // let n = test::black_box(1000000);
            for _i in 1..1000 {
                test::black_box(geohash::encode(point, 12));
            }
        })
    }
}
