use std::borrow::Cow;
use std::io::{BufWriter, Write};

/// Collects contents of a text file.
pub(crate) struct TextBuffer {
    // Use a rope to avoid needlessly copying static strings, of which we have a lot.
    // The lifetime could be generic but this complicates the code and provides little value as of now.
    rope: Vec<Cow<'static, str>>,
}

impl<'a> TextBuffer {
    pub(crate) fn new() -> Self {
        Self { rope: Vec::new() }
    }

    pub(crate) fn add_section<T: Into<Cow<'static, str>>>(&mut self, text: T) {
        self.add_content("\n"); // Add a blank line between existing and new content
        self.add_content(text)
    }

    pub(crate) fn add_content<T: Into<Cow<'static, str>>>(&mut self, text: T) {
        self.rope.push(text.into())
    }

    pub(crate) fn to_writer<W: Write>(&self, writer: W) -> std::io::Result<()> {
        let mut buf_writer = BufWriter::new(writer);
        for segment in &self.rope {
            buf_writer.write(segment.as_bytes())?;
        }
        buf_writer.flush()
    }
}
