fn main() {
    println!("cargo:rustc-link-search=native=/opt/wsb-sysroot/usr/lib");	
    println!("cargo:rustc-link-lib=dylib=wsb_runtime");
    println!("cargo:rerun-if-changed=/opt/wsb-sysroot/usr/lib/libwsb_runtime.so");
}
