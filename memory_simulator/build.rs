use std::env;
use std::path::Path;

fn main() {
    let wrapper_path = "/tmp/memory_simulator/src";
    let ramulator_path = "/tmp/memory_simulator/lib";

    // Verify library files exist
    assert!(
        Path::new(&format!("{}/libwrapper.so", wrapper_path)).exists(),
        "libwrapper.so not found"
    );
    assert!(
        Path::new(&format!("{}/libramulator.so", ramulator_path)).exists(),
        "libramulator.so not found"
    );

    // Set library search paths
    println!("cargo:rustc-link-search=all={}", wrapper_path);
    println!("cargo:rustc-link-search=all={}", ramulator_path);

    // Set LD_LIBRARY_PATH for runtime
    println!(
        "cargo:rustc-env=LD_LIBRARY_PATH={0}:{1}:$LD_LIBRARY_PATH",
        ramulator_path, wrapper_path
    );

    // Add rpath entries
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", ramulator_path);
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", wrapper_path);

    // Set DT_RUNPATH instead of DT_RPATH
    println!("cargo:rustc-link-arg=-Wl,--enable-new-dtags");

    // Link against libraries
    println!("cargo:rustc-link-lib=ramulator");
    println!("cargo:rustc-link-lib=wrapper");

    // Debug information
    println!("cargo:warning=wrapper_path: {}", wrapper_path);
    println!("cargo:warning=ramulator_path: {}", ramulator_path);
}
