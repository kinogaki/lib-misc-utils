#[test]
fn scopes_compile_away() {
    let x = 3;
    profiling::scope!("work");
    profiling::scope!("work", &format!("{x}"));
    profiling::register_thread!("main");
    profiling::finish_frame!();
    assert_eq!(x, 3);
}
