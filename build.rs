use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

fn find_static_lib(start_dir: &Path, lib_name: &str) -> Option<PathBuf> {
    WalkDir::new(start_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy() == lib_name)
        .map(|e| e.path().to_path_buf())
}

fn main() {
    let ncbi_dir = Path::new("vendor/ncbi-vdb");

    // Only rerun if specific files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/ncbi-vdb/libs");
    println!("cargo:rerun-if-changed=vendor/ncbi-vdb/interfaces");
    println!("cargo:rerun-if-changed=vendor/ncbi-vdb/setup/konfigure.perl");
    println!("cargo:rerun-if-changed=vendor/ncbi-vdb/Makefile.env");

    // Check if the library already exists
    if find_static_lib(ncbi_dir, "libncbi-vdb.a").is_none() {
        // Run configure
        let configure_status = Command::new("./configure")
            .current_dir(ncbi_dir)
            .arg("--build-prefix=comp")
            .status()
            .expect("Failed to run configure");

        if !configure_status.success() {
            panic!("Configure failed");
        }

        // Run make with optimal thread count
        let threads = num_cpus::get();
        let make_status = Command::new("make")
            .current_dir(ncbi_dir)
            .arg(format!("-j{}", threads))
            .status()
            .expect("Failed to run make");

        if !make_status.success() {
            panic!("Make failed");
        }
    }

    // Find the static library
    let lib_path =
        find_static_lib(ncbi_dir, "libncbi-vdb.a").expect("Could not find libncbi-vdb.a");

    // Tell cargo about the library
    println!(
        "cargo:rustc-link-search=native={}",
        lib_path.parent().unwrap().display()
    );
    println!("cargo:rustc-link-lib=static=ncbi-vdb");

    // Add the c++ standard library
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
