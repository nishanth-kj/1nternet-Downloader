fn main() {
    println!("cargo:rerun-if-changed=c/fast_ops.c");
    println!("cargo:rerun-if-changed=c/fast_ops.h");

    cc::Build::new()
        .file("c/fast_ops.c")
        .include("c")
        .compile("fast_ops");
}
