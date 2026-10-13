fn main() {
    pkg_config::probe_library("libmypaint").expect("libmypaint not found via pkg-config");
    let brushes_dir = pkg_config::get_variable("mypaint-brushes-2.0", "brushesdir")
        .expect("mypaint-brushes not found via pkg-config");
    println!("cargo:rustc-env=MYPAINT_BRUSHES_DIR={brushes_dir}");
}
