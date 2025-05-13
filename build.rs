extern crate embed_resource;

fn main() {
    // Only run the resource embedding on Windows
    #[cfg(target_os = "windows")]
    {
        // This embeds application metadata in the Windows executable
        embed_resource::compile("windows-resources.rc", &[] as &[&str]);
    }
    
    // These flags ensure static linking for portability on Windows
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:msvcrt");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:msvcrtd");
        
        // Use static CRT
        println!("cargo:rustc-link-arg=-static");
        println!("cargo:rustc-link-search=native=.");
    }
    
    // Set environment variable to help with cross-compilation
    println!("cargo:rustc-env=GMP_MPFR_SYS_CACHE=1");
}
