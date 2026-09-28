#[cfg(jocky_has_object)]
unsafe extern "C" {
    fn jocky_entry() -> i32;
}

#[cfg(jocky_has_object)]
fn main() {
    std::hint::black_box([
        jocky_runtime::jocky_runtime_new as usize,
        jocky_runtime::jocky_runtime_begin as usize,
        jocky_runtime::jocky_runtime_op as usize,
        jocky_runtime::jocky_runtime_finish as usize,
        jocky_runtime::jocky_runtime_free as usize,
    ]);
    std::process::exit(unsafe { jocky_entry() });
}

#[cfg(not(jocky_has_object))]
fn main() {
    eprintln!("build a JOCKY source file with `jocky build` first");
    std::process::exit(1);
}
