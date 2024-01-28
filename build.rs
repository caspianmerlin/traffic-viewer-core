fn main() {
    println!("cargo:rustc-link-lib=static=FSUIPCuser64");
    println!("cargo:rustc-link-search=native=lib/");
}