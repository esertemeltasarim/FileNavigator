extern crate embed_resource;

fn main() {
    // Only run the resource embedding on Windows
    #[cfg(target_os = "windows")]
    {
        // This embeds application metadata in the Windows executable
        embed_resource::compile("windows-resources.rc");
        
        // Enable static linking for Windows
        println!("cargo:rustc-link-arg=-static");
    }
}
