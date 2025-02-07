use std::error::Error;
use vergen::EmitBuilder;

// https://docs.rs/vergen/latest/vergen/struct.EmitBuilder.html#method.emit
fn main() -> Result<(), Box<dyn Error>> {
  static_vcruntime::metabuild();
  EmitBuilder::builder()
    .all_build()
    .all_cargo()
    .all_git()
    .all_rustc()
    .emit()?;
  Ok(())
}
