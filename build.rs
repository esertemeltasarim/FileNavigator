extern crate embed_resource;

fn main() {
    // This embeds application metadata in the Windows executable
    embed_resource::compile("windows-resources.rc", &[] as &[&str]);
    
    // These flags ensure static linking for portability
    println!("cargo:rustc-link-arg=-static");
    println!("cargo:rustc-link-search=native=.");
    println!("cargo:rustc-flags=-Crelocation-model=static");
    println!("cargo:rustc-link-arg=-Bstatic");
}
