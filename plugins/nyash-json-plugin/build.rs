fn main() {
    // Build vendored C shim for yyjson provider (skeleton).
    // This keeps linkage ready without introducing external deps.
    let shim = "c/yyjson_shim.c";
    let yyjson_c = "c/yyjson/yyjson.c";
    let mut b = cc::Build::new();
    let mut need = false;
    if std::path::Path::new(yyjson_c).exists() {
        b.file(yyjson_c);
        println!("cargo:rerun-if-changed={}", yyjson_c);
        need = true;
    }
    if std::path::Path::new(shim).exists() {
        b.file(shim);
        println!("cargo:rerun-if-changed={}", shim);
        need = true;
    }
    if need {
        b.include("c")
            .include("c/yyjson")
            .warnings(false)
            .compile("yyjson_shim");
    }
}
