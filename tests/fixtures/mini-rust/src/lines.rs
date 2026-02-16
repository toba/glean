/// Iterator over lines of a byte slice.
pub struct LineIter<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> LineIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        LineIter { data, pos: 0 }
    }
}

impl<'a> Iterator for LineIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() {
            return None;
        }
        let start = self.pos;
        while self.pos < self.data.len() && self.data[self.pos] != b'\n' {
            self.pos += 1;
        }
        let end = self.pos;
        if self.pos < self.data.len() {
            self.pos += 1; // skip newline
        }
        Some(&self.data[start..end])
    }
}
