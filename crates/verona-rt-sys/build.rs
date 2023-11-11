fn main() {
    let dst = cmake::Config::new("cpp")
        // .build_target("boxcar_bindings")
        .generator("Ninja")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=boxcar_bindings");

    // This is GCC's.
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=atomic");

    // cargo:rustc-link-lib

    // This is LLVM's
    // println!("cargo:rustc-link-lib=static=c++");
}
