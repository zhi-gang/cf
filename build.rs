use std::fs;
use std::path::Path;
fn main()  -> Result<(), Box<dyn std::error::Error>>{
    println!(" - -- --- - -- - - - -  * * * * ** * * * * *  **  *");
    // Path to the source directory containing the config files
    let src_dir = Path::new("src/config");

    // Path to the target directory where the config files will be copied
    let target_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("config");
    println!("target: {:?}", target_dir);
    // Create the target directory if it doesn't exist
    fs::create_dir_all(&target_dir).expect("Failed to create target directory");

    // Copy each file from src_dir to target_dir
    for entry in fs::read_dir(src_dir).expect("Failed to read source directory") {
        let entry = entry.expect("Failed to get entry");
        let file_name = entry.file_name();
        let dest_path = target_dir.join(&file_name);
        fs::copy(entry.path(), dest_path).expect("Failed to copy file");
    }
    Ok(())
}
// fn main() {
//     println!("build");
// }
