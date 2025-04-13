fn main() {
    // This ensures the correct linker script is used
    // Converts the `memory.x` file content to a byte array and links it
    embuild::build::CfgArgs::output_propagated("ESP_IDF").unwrap();
    embuild::build::LinkArgs::output_propagated("ESP_IDF").unwrap();
}
