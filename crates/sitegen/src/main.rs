fn main() -> Result<(), sitegen::BuildError> {
    let config = sitegen::BuildConfig::discover()?;
    let report = sitegen::build(&config)?;
    println!(
        "built {} pages with asset key {}",
        report.page_count, report.cache_key
    );
    Ok(())
}
