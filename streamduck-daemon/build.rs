fn main() {
    if cfg!(windows) {
        println!("cargo:rustc-link-lib=setupapi");
    }
}