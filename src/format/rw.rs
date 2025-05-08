use std::io::{Read, Write};

pub enum InputSource {
    Stdin(std::io::Stdin),
    File(std::fs::File),
}

impl std::io::Read for InputSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InputSource::Stdin(s) => s.read(buf),
            InputSource::File(f) => f.read(buf),
        }
    }
}

pub fn create_or_replace_file(file_name: &String, contents: String) -> std::io::Result<()> {
    let mut file = std::fs::File::create(file_name)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn read_input_to_string(
    buffer: InputSource, string: &mut String,
) -> std::result::Result<usize, std::io::Error> {
    std::io::BufReader::new(buffer).read_to_string(string)
}
