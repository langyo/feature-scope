use anyhow::Result;
use feature_scope::load;

fn main() -> Result<()> {
    load()?;
    Ok(())
}
