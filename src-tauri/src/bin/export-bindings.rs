fn main() {
    vrcx_0::bindings_export::export_bindings().expect("failed to export typescript bindings");
    println!("Generated src/platform/tauri/bindings.ts");
}
