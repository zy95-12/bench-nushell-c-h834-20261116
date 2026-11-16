/// uucore 0.2 uses one utility-specific localizer per thread. Run each utility
/// in its own scope so successive shell commands do not reuse another utility's
/// translations. Error formatting must also happen inside the scope.
pub(crate) fn localized<T: Send>(utility: &str, action: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        scope
            .spawn(move || {
                let _ = uucore::locale::setup_localization(utility);
                action()
            })
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}
