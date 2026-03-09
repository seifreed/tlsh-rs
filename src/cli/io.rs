use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::{TlshBuilder, TlshDigest, TlshError, TlshProfile};

#[derive(Debug)]
pub struct CliContext<'a> {
    stdin_bytes: Option<&'a [u8]>,
    stdin_consumed: bool,
}

impl<'a> CliContext<'a> {
    pub fn new(stdin_bytes: Option<&'a [u8]>) -> Self {
        Self {
            stdin_bytes,
            stdin_consumed: false,
        }
    }

    pub fn load_input(
        &mut self,
        input: &str,
        profile: TlshProfile,
    ) -> Result<TlshDigest, TlshError> {
        if input == "-" || Path::new(input).exists() {
            self.hash_input(input, profile)
        } else {
            TlshDigest::from_encoded(input)
        }
    }

    pub fn hash_input(
        &mut self,
        input: &str,
        profile: TlshProfile,
    ) -> Result<TlshDigest, TlshError> {
        if input == "-" {
            let bytes = self.take_stdin()?;
            hash_bytes(bytes, profile)
        } else {
            hash_file(input, profile)
        }
    }

    fn take_stdin(&mut self) -> Result<&'a [u8], TlshError> {
        if self.stdin_consumed {
            return Err(TlshError::StdinAlreadyConsumed);
        }
        self.stdin_consumed = true;
        self.stdin_bytes.ok_or(TlshError::StdinUnavailable)
    }
}

fn hash_file(path: &str, profile: TlshProfile) -> Result<TlshDigest, TlshError> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Err(TlshError::FileRead(path.to_string())),
    };
    let mut reader = BufReader::new(file);
    let mut builder = TlshBuilder::with_profile(profile);
    let mut buffer = [0u8; 8192];

    loop {
        let read = match reader.read(&mut buffer) {
            Ok(read) => read,
            Err(_) => return Err(TlshError::FileRead(path.to_string())),
        };
        if read == 0 {
            break;
        }
        builder.update(&buffer[..read])?;
    }

    builder.finalize()
}

fn hash_bytes(bytes: &[u8], profile: TlshProfile) -> Result<TlshDigest, TlshError> {
    let mut builder = TlshBuilder::with_profile(profile);
    builder.update(bytes)?;
    builder.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn load_input_parses_digest_when_path_does_not_exist() {
        let mut context = CliContext::new(None);
        let digest = context
            .load_input(
                "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC",
                TlshProfile::standard_t1(),
            )
            .unwrap();
        assert_eq!(
            digest.encoded(),
            "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC"
        );
    }

    #[test]
    fn hash_input_rejects_missing_stdin() {
        let mut context = CliContext::new(None);
        let error = context
            .hash_input("-", TlshProfile::standard_t1())
            .unwrap_err();
        assert_eq!(error, TlshError::StdinUnavailable);
    }

    #[test]
    fn hash_input_rejects_second_stdin_read() {
        let bytes = std::fs::read(fixture("small.txt")).unwrap();
        let mut context = CliContext::new(Some(&bytes));
        let _ = context.hash_input("-", TlshProfile::standard_t1()).unwrap();
        let error = context
            .hash_input("-", TlshProfile::standard_t1())
            .unwrap_err();
        assert_eq!(error, TlshError::StdinAlreadyConsumed);
    }

    #[test]
    fn hash_input_reads_files() {
        let mut context = CliContext::new(None);
        let digest = context
            .hash_input(&fixture("small.txt"), TlshProfile::standard_t1())
            .unwrap();
        assert_eq!(
            digest.encoded(),
            "T1F8A0220C0F8C0023CB880800CA33E88B8F0C022AB302C2008A030300300E8A00C83AAC"
        );
    }

    #[test]
    fn hash_input_reports_missing_file() {
        let mut context = CliContext::new(None);
        let error = context
            .hash_input("definitely-missing-file.bin", TlshProfile::standard_t1())
            .unwrap_err();
        assert_eq!(
            error,
            TlshError::FileRead("definitely-missing-file.bin".to_string())
        );
    }

    #[test]
    fn hash_input_reports_read_errors_after_opening_file() {
        let mut context = CliContext::new(None);
        let directory = env!("CARGO_MANIFEST_DIR");
        let error = context
            .hash_input(directory, TlshProfile::standard_t1())
            .unwrap_err();
        assert_eq!(error, TlshError::FileRead(directory.to_string()));
    }
}
