extern crate embed_resource;

fn main() {
    // This embeds the application icon in the Windows executable
    embed_resource::compile("windows-resources.rc");
}
