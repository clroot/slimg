use std::env;
use std::path::PathBuf;

fn main() {
    let dst = cmake::Config::new("libjxl")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("JPEGXL_ENABLE_TOOLS", "OFF")
        .define("JPEGXL_ENABLE_DOXYGEN", "OFF")
        .define("JPEGXL_ENABLE_MANPAGES", "OFF")
        .define("JPEGXL_ENABLE_BENCHMARK", "OFF")
        .define("JPEGXL_ENABLE_EXAMPLES", "OFF")
        .define("JPEGXL_ENABLE_SJPEG", "OFF")
        .define("JPEGXL_ENABLE_JPEGLI", "OFF")
        .define("JPEGXL_ENABLE_OPENEXR", "OFF")
        .define("JPEGXL_ENABLE_TCMALLOC", "OFF")
        .define("JPEGXL_BUNDLE_LIBPNG", "OFF")
        .define("JPEGXL_ENABLE_SKCMS", "ON")
        .build();

    let lib_dir = dst.join("lib");
    let lib64_dir = dst.join("lib64");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-search=native={}", lib64_dir.display());

    // skcms transform files not included in libjxl's skcms.cmake
    let skcms_src = PathBuf::from("libjxl/third_party/skcms/src");
    let skcms_inc = PathBuf::from("libjxl/third_party/skcms/src");
    cc::Build::new()
        .cpp(true)
        .file(skcms_src.join("skcms_TransformBaseline.cc"))
        .include(&skcms_inc)
        .flag_if_supported("-Wno-psabi")
        .compile("skcms_transform");

    // libjxl core (encoder + decoder are bundled into libjxl.a)
    println!("cargo:rustc-link-lib=static=jxl");
    println!("cargo:rustc-link-lib=static=jxl_cms");

    // libjxl vendored dependencies
    println!("cargo:rustc-link-lib=static=hwy");
    println!("cargo:rustc-link-lib=static=brotlienc");
    println!("cargo:rustc-link-lib=static=brotlidec");
    println!("cargo:rustc-link-lib=static=brotlicommon");

    // C++ standard library
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    match target_os.as_str() {
        "macos" | "ios" => println!("cargo:rustc-link-lib=c++"),
        "windows" => {} // MSVC links C++ runtime automatically
        _ => println!("cargo:rustc-link-lib=stdc++"),
    }

    // bindgen
    let include_dir = dst.join("include");
    let src_include = PathBuf::from("libjxl/lib/include");
    let target = env::var("TARGET").unwrap();

    let bindings = bindgen::Builder::default()
        .header(src_include.join("jxl/encode.h").to_str().unwrap())
        .header(src_include.join("jxl/decode.h").to_str().unwrap())
        .header(src_include.join("jxl/types.h").to_str().unwrap())
        .header(src_include.join("jxl/codestream_header.h").to_str().unwrap())
        .header(src_include.join("jxl/color_encoding.h").to_str().unwrap())
        .clang_arg(format!("-I{}", src_include.display()))
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_arg(format!("--target={target}"))
        // Encoder functions
        .allowlist_function("JxlEncoderCreate")
        .allowlist_function("JxlEncoderDestroy")
        .allowlist_function("JxlEncoderReset")
        .allowlist_function("JxlEncoderSetBasicInfo")
        .allowlist_function("JxlEncoderSetColorEncoding")
        .allowlist_function("JxlEncoderFrameSettingsCreate")
        .allowlist_function("JxlEncoderSetFrameDistance")
        .allowlist_function("JxlEncoderSetFrameLossless")
        .allowlist_function("JxlEncoderFrameSettingsSetOption")
        .allowlist_function("JxlEncoderAddImageFrame")
        .allowlist_function("JxlEncoderCloseInput")
        .allowlist_function("JxlEncoderProcessOutput")
        .allowlist_function("JxlEncoderDistanceFromQuality")
        .allowlist_function("JxlColorEncodingSetToSRGB")
        // Decoder functions
        .allowlist_function("JxlDecoderCreate")
        .allowlist_function("JxlDecoderDestroy")
        .allowlist_function("JxlDecoderReset")
        .allowlist_function("JxlDecoderSubscribeEvents")
        .allowlist_function("JxlDecoderSetInput")
        .allowlist_function("JxlDecoderCloseInput")
        .allowlist_function("JxlDecoderProcessInput")
        .allowlist_function("JxlDecoderGetBasicInfo")
        .allowlist_function("JxlDecoderImageOutBufferSize")
        .allowlist_function("JxlDecoderSetImageOutBuffer")
        .allowlist_function("JxlDecoderReleaseInput")
        // Encoder types
        .allowlist_type("JxlEncoderStatus")
        .allowlist_type("JxlEncoderFrameSettingId")
        .allowlist_type("JxlEncoder")
        .allowlist_type("JxlEncoderFrameSettings")
        // Decoder types
        .allowlist_type("JxlDecoder")
        .allowlist_type("JxlDecoderStatus")
        // Shared types
        .allowlist_type("JxlBasicInfo")
        .allowlist_type("JxlPixelFormat")
        .allowlist_type("JxlDataType")
        .allowlist_type("JxlEndianness")
        .allowlist_type("JxlColorEncoding")
        .generate()
        .expect("failed to generate libjxl bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("failed to write bindings");
}
