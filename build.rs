use std::fs::File;
use std::path::Path;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("payload.tar.gz");

    // Watch installer_payload directory
    println!("cargo:rerun-if-changed=installer_payload");

    let payload_dir = Path::new("installer_payload");
    if payload_dir.exists() {
        let tar_gz = File::create(&dest_path).expect("failed to create payload.tar.gz");
        let enc = GzEncoder::new(tar_gz, Compression::best());
        let mut tar = Builder::new(enc);

        tar.append_dir_all("installer_payload", payload_dir)
            .expect("failed to pack installer_payload directory");
        tar.finish().expect("failed to finish tar archive");
    } else {
        // Fallback dummy file if building in an environment without payload
        let dummy = File::create(&dest_path).expect("failed to create empty dummy payload");
        let enc = GzEncoder::new(dummy, Compression::default());
        let mut tar = Builder::new(enc);
        tar.finish().expect("failed to finish empty tar");
    }
}
