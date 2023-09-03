use std::{
    env,
    fs::File,
    io::{Read, Result, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

/// # Return
/// Absolute path to the top of the git repo hosting this project
pub fn get_project_root() -> Result<PathBuf> {
    // let project_root_dir_output = std::process::Command::new("git")
    //     .arg("rev-parse")
    //     .arg("--show-toplevel")
    //     .output()?
    //     .stdout;
    let project_root_dir_output = std::process::Command::new("cargo")
        .arg("locate-project")
        .arg("--message-format")
        .arg("plain")
        .output()?
        .stdout;
    let output = String::from_utf8(project_root_dir_output).unwrap();

    let mut project_root_dir = PathBuf::from_str(output.as_str()).unwrap();
    project_root_dir.pop();
    Ok(project_root_dir)
}

/// Allows generated file to be include!(concat!(env!("OUT_DIR"), "/generated_protos/foo.rs"));
/// Work around inspired by
/// https://github.com/cartographer-project/point_cloud_viewer/blob/440d875f12e32dff6107233f24b5a02cf28776dc/point_viewer_proto_rust/build.rs#L33
/// https://github.com/stepancheg/rust-protobuf/issues/117
/// https://github.com/rust-lang/rust/issues/18810.
///
/// We open the file, add 'mod proto { }' around the contents and write it back. This allows us
/// to include! the file in lib.rs and have a proper proto module.
/// Name each module with the name of the .proto file to avoid collisions.
fn wrap_output_rust_proto(proto_path: &PathBuf) {
    let mut contents = String::new();
    File::open(proto_path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let proto_rs_filename = proto_path.as_path().with_extension("");

    let proto_module_name = proto_rs_filename
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap();

    let new_contents = format!("pub mod {} {{\n{}\n}}", proto_module_name, contents);

    File::create(proto_path)
        .unwrap()
        .write_all(new_contents.as_bytes())
        .unwrap();
}

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();

    let proto_generator_root_dir = get_project_root()?;
    let root_dir = proto_generator_root_dir.parent().unwrap();

    protobuf_codegen::Codegen::new()
        .includes([root_dir.join("protobufs")])
        .inputs([root_dir.join("protobufs/notices.proto")])
        .cargo_out_dir("generated_protos")
        .run()
        .unwrap();

    wrap_output_rust_proto(&Path::new(&out_dir).join("generated_protos/notices.rs"));

    Ok(())
}
