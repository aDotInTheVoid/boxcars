fn main() {
    let mut cmake_build = cmake::Config::new("cpp");

    if has_ninja() {
        cmake_build.generator("Ninja");
    }

    if cfg!(feature = "systematic_testing") {
        cmake_build.define("USE_SYSTEMATIC_TESTING", "ON");
    }

    let dst = cmake_build.build();

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=boxcar_bindings");

    // This is GCC's.
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=atomic");

    // This is LLVM's
    // println!("cargo:rustc-link-lib=static=c++");
}

fn has_ninja() -> bool {
    std::process::Command::new("ninja")
        .arg("--version")
        .output()
        .is_ok()
}
