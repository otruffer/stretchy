extern crate embed_resource;

fn main() {
    embed_resource::compile("tray-resources.rc", embed_resource::NONE);
}