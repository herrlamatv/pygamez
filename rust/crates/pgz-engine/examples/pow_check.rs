use std::io::Read;
fn main() {
    let p = std::env::args().nth(1).unwrap();
    let mut s = String::new();
    std::fs::File::open(p).unwrap().read_to_string(&mut s).unwrap();
    let (mut a, mut b, mut c, mut d, mut e) = (0, 0, 0, 0, 0);
    for l in s.lines() {
        let v: Vec<f64> = l.split(' ').map(|z| z.parse().unwrap()).collect();
        if v[0].powf(2.0) != v[1] { a += 1; }
        if v[0].powf(3.0) != v[2] { b += 1; }
        if v[0] * v[0] != v[1] { c += 1; }
        if v[0].abs().powf(0.5) != v[3] { d += 1; }
        if v[0].powi(3) != v[2] { e += 1; }
    }
    println!("powf2 {} powf3 {} mul2 {} powf05 {} powi3 {}", a, b, c, d, e);
}
