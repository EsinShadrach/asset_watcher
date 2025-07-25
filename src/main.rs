use heck::ToLowerCamelCase;
use notify::{recommended_watcher, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let project_root = env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let assets_images = project_root.join("assets/images");
    let assets_svgs = project_root.join("assets/svgs");
    let generated_file = project_root.join("lib/generated/assets.dart");

    generate_assets_file(&assets_images, &assets_svgs, &generated_file);

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = recommended_watcher(move |res| {
        tx.send(res).unwrap();
    })
    .unwrap();

    watcher
        .watch(&assets_images, RecursiveMode::Recursive)
        .unwrap();
    watcher
        .watch(&assets_svgs, RecursiveMode::Recursive)
        .unwrap();

    println!("Watching for changes in assets/images and assets/svgs...");

    loop {
        match rx.recv() {
            Ok(_) => {
                println!("Changes detected, regenerating assets file...");
                generate_assets_file(&assets_images, &assets_svgs, &generated_file);
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn generate_assets_file(images_dir: &Path, svgs_dir: &Path, output_file: &Path) {
    let mut file_content = String::new();

    file_content.push_str(
        "class ImageAssets {
",
    );
    for entry in WalkDir::new(images_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap();
            if file_name == ".DS_Store" {
                continue;
            }
            let asset_name =
                ToLowerCamelCase::to_lower_camel_case(file_name.split('.').next().unwrap());
            let asset_path = Path::new("assets/images").join(file_name);
            file_content.push_str(&format!(
                "  static const String {} = \"{}\";\n",
                asset_name,
                asset_path.to_str().unwrap()
            ));
        }
    }
    file_content.push_str(
        "} 
",
    );

    file_content.push_str(
        "class SvgAssets {
",
    );
    for entry in WalkDir::new(svgs_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_str().unwrap();
            if file_name == ".DS_Store" {
                continue;
            }
            let asset_name =
                ToLowerCamelCase::to_lower_camel_case(file_name.split('.').next().unwrap());
            let asset_path = Path::new("assets/svgs").join(file_name);
            file_content.push_str(&format!(
                "  static const String {} = \"{}\";\n",
                asset_name,
                asset_path.to_str().unwrap()
            ));
        }
    }
    file_content.push_str(
        "}
",
    );

    let mut file = fs::File::create(output_file).unwrap();
    file.write_all(file_content.as_bytes()).unwrap();
}
