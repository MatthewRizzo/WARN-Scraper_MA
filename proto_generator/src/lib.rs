// Allows including via proto_generator::export
// The build script names each module the same name as the <name>.proto
include!(concat!(env!("OUT_DIR"), "/generated_protos/notices.rs"));
