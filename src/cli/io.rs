use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::internal::constants::MAX_DATA_LENGTH;
use crate::{TlshBuilder, TlshDigest, TlshError, TlshProfile, hash_bytes_with_profile};

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

    #[allow(clippy::question_mark)]
    pub fn hash_input(
        &mut self,
        input: &str,
        profile: TlshProfile,
    ) -> Result<TlshDigest, TlshError> {
        if input != "-" {
            return hash_file(input, profile);
        }
        if self.stdin_consumed {
            return Err(TlshError::StdinAlreadyConsumed);
        }
        self.stdin_consumed = true;
        let bytes = stdin_bytes_or_err(self.stdin_bytes)?;
        hash_bytes_with_profile(bytes, profile)
    }
}

fn hash_file(path: &str, profile: TlshProfile) -> Result<TlshDigest, TlshError> {
    let file = open_file_or_err(path)?;
    let too_large = file
        .metadata()
        .ok()
        .is_some_and(|metadata| metadata.len() > MAX_DATA_LENGTH);
    if too_large {
        return Err(TlshError::DataTooLong);
    }
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
        update_builder_chunk(&mut builder, &buffer[..read])
            .expect("file length is validated before chunk processing");
    }

    builder.finalize()
}

fn stdin_bytes_or_err(stdin_bytes: Option<&[u8]>) -> Result<&[u8], TlshError> {
    match stdin_bytes {
        Some(bytes) => Ok(bytes),
        None => Err(TlshError::StdinUnavailable),
    }
}

fn open_file_or_err(path: &str) -> Result<File, TlshError> {
    match File::open(path) {
        Ok(file) => Ok(file),
        Err(_) => Err(TlshError::FileRead(path.to_string())),
    }
}

fn update_builder_chunk(builder: &mut TlshBuilder, chunk: &[u8]) -> Result<(), TlshError> {
    builder.update(chunk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture(name: &str) -> String {
        format!("{}/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tlsh-rs-{name}-{nanos}-{}", std::process::id()))
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

    #[test]
    fn hash_input_rejects_sparse_files_over_max_length() {
        let path = unique_temp_path("too-large");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.set_len(MAX_DATA_LENGTH + 1).unwrap();

        let mut context = CliContext::new(None);
        let error = context
            .hash_input(path.to_str().unwrap(), TlshProfile::standard_t1())
            .unwrap_err();
        assert_eq!(error, TlshError::DataTooLong);

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn metadata_size_check_uses_real_file_length() {
        let path = unique_temp_path("metadata");
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        assert_eq!(file.metadata().unwrap().len(), 0);
        file.set_len(MAX_DATA_LENGTH + 1).unwrap();
        assert!(file.metadata().unwrap().len() > MAX_DATA_LENGTH);
        drop(file);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn io_helpers_cover_success_and_error_paths() {
        let bytes = b"abc";
        assert_eq!(stdin_bytes_or_err(Some(bytes)).unwrap(), bytes);
        assert_eq!(
            stdin_bytes_or_err(None).unwrap_err(),
            TlshError::StdinUnavailable
        );

        let opened = open_file_or_err(&fixture("small.txt")).unwrap();
        assert!(opened.metadata().unwrap().is_file());

        let error = open_file_or_err("definitely-missing-file.bin").unwrap_err();
        assert_eq!(
            error,
            TlshError::FileRead("definitely-missing-file.bin".to_string())
        );
    }

    #[test]
    fn update_builder_chunk_reports_data_too_long_without_reading_input() {
        let oversized = unsafe {
            std::slice::from_raw_parts(
                std::ptr::NonNull::<u8>::dangling().as_ptr(),
                (MAX_DATA_LENGTH + 1) as usize,
            )
        };
        let mut builder = TlshBuilder::new();
        let error = update_builder_chunk(&mut builder, oversized).unwrap_err();
        assert_eq!(error, TlshError::DataTooLong);
    }
}
