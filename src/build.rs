use vergen_gitcl::{Emitter, GitclBuilder, RustcBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .add_instructions(
            &RustcBuilder::default()
                .host_triple(true)
                .semver(true)
                .build()?,
        )?
        .add_instructions(
            &GitclBuilder::default()
                .commit_date(true)
                .sha(true)
                .build()?,
        )?
        .emit()?;

    Ok(())
}
