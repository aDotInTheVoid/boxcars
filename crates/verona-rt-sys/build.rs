const FEATURE_ASAN: bool = cfg!(feature = "sanitizer_address");
const FEATURE_TSAN: bool = cfg!(feature = "sanitizer_thread");

fn assert_both(z_flag: bool, feature: bool, san_name: &str) {
    match (z_flag, feature) {
        (true, true) | (false, false) => {}
        (true, false) => {
            panic!("`-Zsanitizer={san_name}` set, but don't have `--features=sanitizer_{san_name}`")
        }
        (false, true) => {
            panic!("`--features=sanitizer_{san_name}` set, but don't have `-Zsanitizer={san_name}`")
        }
    }
}

fn main() {
    let flags = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();
    let mut flags = flags.split('\u{1f}');

    let mut z_sanitizer_address = false;
    let mut z_sanitizer_thread = false;

    while let Some(flag) = flags.next() {
        match flag {
            "-Zsanitizer=thread" => z_sanitizer_thread = true,
            "-Zsanitizer=address" => z_sanitizer_address = true,
            "" => {}
            other => panic!("unknown flag {other:?}"),
        }
    }

    assert_both(z_sanitizer_address, FEATURE_ASAN, "address");
    assert_both(z_sanitizer_thread, FEATURE_TSAN, "thread");

    let mut cmake_build = cmake::Config::new("cpp");

    if has_ninja() {
        cmake_build.generator("Ninja");
    }

    if cfg!(feature = "systematic_testing") {
        cmake_build.define("USE_SYSTEMATIC_TESTING", "ON");
    }
    if cfg!(feature = "flight_recorder") {
        cmake_build.define("USE_CRASH_LOGGING", "ON");
    }
    if cfg!(feature = "snmalloc_tracing") {
        cmake_build.define("SNMALLOC_TRACING", "ON");
    }

    match (FEATURE_ASAN, FEATURE_TSAN) {
        (false, false) => {}
        (true, false) => _ = cmake_build.define("SANITIZER", "address"),
        (false, true) => _ = cmake_build.define("SANITIZER", "thread"),
        (true, true) => panic!("Can't use both asan & tsan"),
    }

    // https://github.com/aDotInTheVoid/boxcars/issues/1#issuecomment-1812070337
    cmake_build.define("VERONA_RT_ONLY_HEADER_LIBRARY", "ON");

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
