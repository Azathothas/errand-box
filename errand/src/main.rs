//! The `errand` binary. The command line and the daemon are the library's,
//! so errand-box can run the same code under the name `errand`.

fn main() -> std::process::ExitCode {
    errand::main()
}
