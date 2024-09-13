use std::path::PathBuf;

#[cfg(feature = "selfinit")]
/// Decompresses a folder from the bits of a .tar.xz file
pub(crate) fn decompress_folder(compressed_folder: &[u8], output_folder: PathBuf) {
    let decompressed_folder = lzma::decompress(compressed_folder).unwrap();
    let mut archive = tar::Archive::new(decompressed_folder.as_slice());
    archive.unpack(output_folder).unwrap();
}
