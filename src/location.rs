#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    filename: String,
    rol: usize,
    col: usize
}

impl Location {
    pub fn new(filename: String, rol: usize, col: usize) -> Self {
        Self {
            filename,
            rol,
            col,
        }
    }

    pub fn set_filename(&mut self, s: String) {
        self.filename = s;
    }

    pub fn filename(&self) -> &str {
        &self.filename.as_str()
    }

    pub fn rol(&self) -> usize {
        self.rol
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "file: {:?}, rol: {}, col: {}", self.filename(), self.rol(), self.col())
    }
}