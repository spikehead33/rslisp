mod location;
mod evaluator;
mod lexer;
mod parser;

use lexer::tokenize;

fn main() -> std::io::Result<()> {
    // Testing file read operation
    let fname = std::env::args().nth(1).unwrap();
    let content = std::fs::read_to_string(fname.as_str())?;

    let tokens = tokenize(fname.as_str(), content.as_str());
    Ok(())
}
