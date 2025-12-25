use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

pub(crate) fn read_bytes(path: &str) -> Vec<u8> {
    let file = File::open(path).expect("Cannot open file");
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    buffer
}

pub(crate) fn write_bytes(bytes: &[u8], path: &str) -> Result<(), std::io::Error> {
    let file = File::create(path).expect("failed to create file");
    let mut writer = BufWriter::new(file);

    writer.write_all(bytes).expect("failed to write buffer");
    writer.flush().expect("failed to flush buffer");
    Ok(())
}

pub(crate) fn binary_file_path(directory: &str, filename: &str) -> String {
    let directory = Path::new(directory);
    let graph_file = directory.join(filename);
    graph_file.into_os_string().into_string().unwrap()
}
