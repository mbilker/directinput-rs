fn main() {
    // `dxguid` is needed for many of the included GUID definitions. MSVC
    // reports undefined external symbols while linking without this.
    println!("cargo:rustc-link-lib=dylib=dxguid");
}
