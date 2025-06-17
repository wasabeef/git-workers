use git_workers::utils;

#[test]
fn test_print_functions_dont_panic() {
    // These functions print to stdout, so we just ensure they don't panic
    utils::print_progress("Testing progress");
    utils::print_success("Testing success");
    utils::print_error("Testing error");
}
