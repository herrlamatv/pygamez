//! Vergleicht `py::hypot` mit Werten aus CPython (Datei als Argument).
fn main() {
    let pfad = std::env::args().nth(1).expect("usage: hypot_check <file>");
    let text = std::fs::read_to_string(pfad).expect("cannot read file");
    let (mut gut, mut c_gut, mut n) = (0, 0, 0);
    for z in text.lines() {
        let v: Vec<f64> = z.split(' ').map(|x| x.parse().unwrap()).collect();
        n += 1;
        if pgz_engine::py::hypot(v[0], v[1]).to_bits() == v[2].to_bits() {
            gut += 1;
        }
        if v[0].hypot(v[1]).to_bits() == v[2].to_bits() {
            c_gut += 1;
        }
    }
    println!("py::hypot {gut}/{n} gleich, f64::hypot {c_gut}/{n} gleich");
}
