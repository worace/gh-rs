#![feature(float_extras)]
extern crate geo_types;

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

fn widen(mut low32: u64) -> u64 {
  low32 |= low32 << 16; low32 &= 0x0000ffff0000ffff;
  low32 |= low32 << 8;  low32 &= 0x00ff00ff00ff00ff;
  low32 |= low32 << 4;  low32 &= 0x0f0f0f0f0f0f0f0f;
  low32 |= low32 << 2;  low32 &= 0x3333333333333333;
  low32 |= low32 << 1;  low32 &= 0x5555555555555555;
  low32
}


const mult: f64 = -(0x80000000 as f64);

pub fn encode(point: Coordinate<f64>) -> u64 {
    let biasedLon = (point.x + 180.0) / 360.0;
    let biasedLat = (point.y + 90.0) / 180.0;
    debug_float(biasedLon);
    debug_float(biasedLat);

    let latBits: u64 = ((biasedLat * mult) as u64) & 0x7fffffff;
    let lonBits: u64 = ((biasedLon * mult) as u64) & 0x7fffffff;

    // println!("{}", mult);
    println!("{}", lonBits);
    println!("{}", latBits);
    println!("{:#066b}", lonBits);
    println!("{:#066b}", latBits);

    let latWide = widen(latBits);
    let lonWide = widen(lonBits);
    println!("{:#066b}", lonWide);
    println!("{:#066b}", latWide);

    let encoded = lonWide | (latWide >> 1);
    println!("****************8");
    println!("{:#066b}", encoded);
    encoded
}

#[cfg(test)]
mod tests {
    use geo_types::Coordinate;
    use encode;
    #[test]
    fn encoding() {
        let point = Coordinate {
            x: 112.5584f64,
            y: 37.8324f64,
        };
        assert_eq!(encode(point), 2081272275720008449);
    }
}
