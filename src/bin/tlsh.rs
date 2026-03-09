use std::process::ExitCode;

fn main() -> ExitCode {
    tlsh_rs::cli::run_with_io(
        std::env::args().skip(1).collect(),
        &mut std::io::stdin(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_is_a_thin_non_successful_wrapper_under_test_harness_args() {
        assert_ne!(main(), ExitCode::SUCCESS);
    }
}
