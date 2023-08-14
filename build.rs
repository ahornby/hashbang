use std::env::var;

fn main() {
    println!("cargo:rustc-env=TARGET={}", var("TARGET").unwrap())
}
