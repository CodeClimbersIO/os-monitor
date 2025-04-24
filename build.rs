use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:warning=Build script starting...");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    println!("cargo:info=Target OS: {}", target_os);

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    println!("cargo:warning=Manifest dir: {}", manifest_dir.display());

    if target_os == "macos" {
        println!("cargo:info=Building for macOS...");

        // Define all source files
        let source_files = [
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("Monitor.m"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("AccessibilityUtils.m"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("Application.m"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("Blocker.m"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("AccessibilityElement.m"),
            manifest_dir.join("bindings").join("macos").join("UI.m"),
        ];

        // Define all header files (for dependency tracking)
        let header_files = [
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("Monitor.h"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("Application.h"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("WindowUtils.h"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("Blocker.h"),
            manifest_dir
                .join("bindings")
                .join("macos")
                .join("AccessibilityElement.h"),
            manifest_dir.join("bindings").join("macos").join("UI.h"),
        ];

        let out_dir = std::env::var("OUT_DIR").unwrap();
        let out_path = PathBuf::from(out_dir);
        let include_dir = source_files[0].parent().unwrap();

        println!("cargo:info=Source files: {:?}", source_files);
        println!("cargo:warning=Output directory: {}", out_path.display());

        // Get the target architecture
        let target_arch =
            env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "aarch64".to_string());
        println!("cargo:info=Target architecture: {}", target_arch);

        // Add architecture-specific flags for cross-compilation
        let mut arch_flags: Vec<&str> = Vec::new();
        match target_arch.as_str() {
            "x86_64" => {
                arch_flags.push("-arch");
                arch_flags.push("x86_64");
            }
            "aarch64" => {
                arch_flags.push("-arch");
                arch_flags.push("arm64");
            }
            _ => {}
        };
        println!("cargo:info=Using architecture flags: {:?}", arch_flags);

        // Build the Objective-C code using clang
        println!("cargo:info=Compiling Objective-C code...");
        let mut clang_args = vec![
            "-fobjc-arc",
            "-fmodules",
            "-framework",
            "Cocoa",
            "-dynamiclib",
            "-install_name",
            "@rpath/libMacMonitor.dylib",
        ];

        // Add architecture flags if any
        clang_args.extend(arch_flags);

        let status = std::process::Command::new("clang")
            .args(&clang_args)
            // Add all source files as separate arguments
            .args(source_files.iter().map(|p| p.to_str().unwrap()))
            .args(&[
                "-I",
                include_dir.to_str().unwrap(),
                "-o",
                out_path.join("libMacMonitor.dylib").to_str().unwrap(),
            ])
            .status()
            .expect("Failed to execute clang command");

        if !status.success() {
            panic!("Objective-C compilation failed");
        }

        println!("cargo:warning=Linking to path: {}", out_path.display());
        println!("cargo:info=Setting up library paths...");
        println!("cargo:rustc-link-search=native={}", out_path.display());

        for file in source_files.iter().chain(header_files.iter()) {
            println!("cargo:rerun-if-changed={}", file.display());
        }
    } else if target_os == "windows" {
        println!("cargo:info=Building for Windows...");

        let source_path = manifest_dir
            .join("bindings")
            .join("windows_monitor")
            .join("windows_monitor")
            .join("monitor.c");
        let output_dir = manifest_dir
            .join("bindings")
            .join("windows_monitor")
            .join("windows_monitor")
            .join("release");

        println!("cargo:info=Source path: {}", source_path.display());
        println!("cargo:info=Output directory: {}", output_dir.display());

        std::fs::create_dir_all(&output_dir).unwrap();

        // Compile the C code
        println!("cargo:info=Compiling C code...");
        cc::Build::new()
            .file(&source_path)
            .static_flag(false) // Not a static library
            .out_dir(&output_dir) // Specify where to put the output
            .compile("WindowsMonitor");

        println!("cargo:info=Setting up library paths...");
        println!("cargo:rustc-link-search=native={}", output_dir.display());
        println!("cargo:rustc-link-lib=WindowsMonitor");

        println!("cargo:rustc-link-lib=user32");

        // Tell Cargo to rerun if our source changes
        println!("cargo:rerun-if-changed={}", source_path.display());
        println!("cargo:warning=Build script completed successfully");
    }
}
