use std::env;
use std::fs;
use chrono::Utc;

fn main() {
    // Get version from `Cargo.toml`
    let version = env::var("CARGO_PKG_VERSION").expect("Failed to get package version from Cargo.toml");

    // Debug output
    println!("cargo:warning=Setting VERSION to {}", version);

    // Generate a date string for the build
    let date = Utc::now().format("%Y%m%d").to_string();

    // Track the number of builds for the day
    let build_count_file = format!("target/build_count_{}", date);
    let build_number = match fs::read_to_string(&build_count_file) {
        Ok(count) => count.trim().parse::<u32>().unwrap_or(0) + 1,
        Err(_) => 1,
    };

    // Save the updated build count
    fs::write(&build_count_file, build_number.to_string()).expect("Failed to write build count");

    // Construct the full build version string
    let build_info = format!("{}_{}", date, build_number);

    // Debug output
    println!("cargo:warning=Setting BUILD_NUMBER to {}", build_info);

    // Set environment variables to pass to Cargo during compilation
    println!("cargo:rustc-env=VERSION={}", version);
    println!("cargo:rustc-env=BUILD_NUMBER={}", build_info);
}
