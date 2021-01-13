use std::env;
use std::path::{Path, PathBuf};


#[cfg(feature = "vendored-cmake")]
fn build_libcassandra() -> PathBuf {
    let z_root_dir = env::var("DEP_Z_ROOT")
        .map(PathBuf::from)
        .expect("Missing vendored libz"); // FIXME(don't unwrap)

    let uv_root_dir = env::var("DEP_UV_ROOT")
        .map(PathBuf::from)
        .expect("DEP_UV_ROOT env var is not set. Was libuv vendored properly?");

    let uv_include_dir = env::var("DEP_UV_INCLUDE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            eprintln!("Missing DEP_UV_INCLUDE, falling back to DEP_UV_ROOT");
            uv_root_dir.join("include")
        });

    let ssl_root_dir = env::var("DEP_OPENSSL_ROOT")
        .map(PathBuf::from)
        .expect("Missing vendored libssl"); // FIXME(don't unwrap)

    let mut config = cmake::Config::new("cpp-driver");

    config
        .define("CASS_USE_OPENSSL", "1")
        .define("CASS_USE_ZLIB", "1")
        .define("LIBUV_ROOT_DIR", &uv_root_dir)
        .define("LIBUV_INCLUDE_DIR", &uv_include_dir)
        .define("ZLIB_ROOT", &z_root_dir)
        .define("OPENSSL_ROOT_DIR", &ssl_root_dir)
        .define("CASS_BUILD_STATIC", "1")
        .define("CASS_BUILD_SHARED", "1")
        .register_dep("z")
        .register_dep("openssl")
        .register_dep("uv")
    ;

    let dst = config.build();
    eprintln!("Built libcassandra in dir `{}`", dst.display());
    dst
}

#[cfg(feature = "vendored-cmake")]
fn vendor_and_link_libcassandra() {
    let install_dir = build_libcassandra();

    // link libstdc++
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");
    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-link-lib=dylib=stdc++");

    println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=uv");
    println!("cargo:rustc-link-search=native={}", install_dir.join("lib").display());
    println!("cargo:rustc-link-lib=static=cassandra_static");
}

fn link_libcassandra_dynamic() {
    if let Some(datastax_dir) = option_env!("CASSANDRA_SYS_LIB_PATH") {
        for p in datastax_dir.split(";") {
            println!("cargo:rustc-link-search={}", p);
        }
    }

    println!("cargo:rustc-flags=-l dylib=cassandra");
    println!("cargo:rustc-flags=-l dylib=crypto");
    println!("cargo:rustc-flags=-l dylib=ssl");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-flags=-l dylib=c++");
    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-flags=-l dylib=stdc++");
    println!("cargo:rustc-flags=-l dylib=uv");
    println!("cargo:rustc-link-search={}", "/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search={}", "/usr/local/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search={}", "/usr/local/lib64");
    println!("cargo:rustc-link-search={}", "/usr/local/lib");
    println!("cargo:rustc-link-search={}", "/usr/lib64/");
    println!("cargo:rustc-link-search={}", "/usr/lib/");
    println!("cargo:rustc-link-search={}", "/usr/local/opt/openssl/lib");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if ! cfg!(feature = "vendored-cmake") {
        link_libcassandra_dynamic()
    } else {
        #[cfg(feature = "vendored-cmake")]
        vendor_and_link_libcassandra()
    }
}
