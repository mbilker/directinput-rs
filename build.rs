fn main() {
    // `winapi`'s link definition seems enough with MSVC, but not with MinGW.
    println!("cargo:rustc-link-lib=dylib=dinput8");

    // `dxguid` is needed for many of the included GUID definitions. MSVC
    // reports undefined external symbols while linking without this.
    println!("cargo:rustc-link-lib=dylib=dxguid");
}
