fn main() {
    println!("cargo:rustc-link-search=native=build");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    println!("cargo:rustc-link-search=native={manifest_dir}/build");
println!("cargo:rustc-link-lib=static=peroxide");
}
