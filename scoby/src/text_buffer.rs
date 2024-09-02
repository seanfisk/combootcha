use std::io::{Write, BufWriter};
use std::borrow::Cow;

/// Collects contents of a text file.
pub(crate) struct TextBuffer<'a> {
    rope: Vec<Cow<'a, str>>,
}

impl<'a> TextBuffer<'a> {
    pub(crate) fn new() -> Self {
        Self { rope: Vec::new() }
    }

    fn add_section<T: Into<Cow<'a, str>>>(&mut self, text: T) {
        self.add_content("\n"); // Add a blank line between existing and new content
        self.add_content(text)
    }

    fn add_content<T: Into<Cow<'a, str>>>(&mut self, text: T) {
        self.rope.push(text.into())
    }

    fn to_writer<W: Write>(&self, writer: W) -> std::io::Result<()> {
        let mut buf_writer = BufWriter::new(writer);
        for segment in &self.rope {
            buf_writer.write(segment.as_bytes())?;
        }
        buf_writer.flush()
    }
}

