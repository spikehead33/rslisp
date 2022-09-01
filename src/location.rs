#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    filename: Option<String>,
    rol: usize,
    col: usize
}

impl Location {
    pub fn new(filename: Option<String>, rol: usize, col: usize) -> Self {
        Self {
            filename,
            rol,
            col,
        }
    }

    pub fn set_filename(&mut self, s: String) {
        self.filename = Some(s);
    }

    pub fn filename(&self) -> Option<&str> {
        if let Some(s) = &self.filename {
            Some(s.as_str())
        } else {
            None
        }
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