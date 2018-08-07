extern crate geo_types;

pub use geo_types::Coordinate;

pub fn encode(point: Coordinate<f64>) -> i64 {
    0
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
        assert_eq!(encode(point), 0);
    }
}
