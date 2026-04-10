use std::io::BufRead;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

fn consume_while<R, P>(reader: &mut R, predicate: P) -> Result<(), std::io::Error>
where
    R: BufRead,
    P: for<'a> FnMut(&'a &u8) -> bool,
{
    // TODO: Loop to fill buffer and consume while the predicate holds true.
    let buf = reader.fill_buf()?;
    let consumed = buf.iter().take_while(predicate).count();
    reader.consume(consumed);
    Ok(())
}

fn skip_ws<R: BufRead>(reader: &mut R) -> Result<(), ParseError> {
    Ok(consume_while(reader, |b| b" \t\n\r".contains(b))?)
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::io::{Cursor, Read};

    use test_case::test_case;

    use crate::json_parser::*;

    #[test_case("   ", ""; "only spaces")]
    #[test_case("\t\t\t", ""; "only horizontal tabs")]
    #[test_case("\n\n\n", ""; "only line feeds")]
    #[test_case("\r\r\r", ""; "only carriage returns")]
    #[test_case("  \t  \r  \n", ""; "all at once")]
    #[test_case("null \t", "null \t"; "null then spaces")]
    #[test_case(" \tnull", "null"; "spaces then null")]
    fn test_skip_ws(json_string: &str, after_skipped: &str) {
        let mut reader = Cursor::new(json_string);
        let mut remaining_in_reader = String::new();

        assert!(skip_ws(&mut reader).is_ok());
        assert_eq!(
            reader.read_to_string(&mut remaining_in_reader).unwrap(),
            after_skipped.len()
        );
        assert_eq!(remaining_in_reader, after_skipped);
    }
}
