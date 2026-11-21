/*!
IR passes that backends run on a module before writing it.

Each backend chooses the passes it needs and their options. The passes modify
one copy of the module in place, after which the module is compacted, which
reorders function expressions that the passes appended, and revalidated, so
the backend always writes a module that matches its [`ModuleInfo`].
*/
#![cfg_attr(
    not(any(glsl_out, hlsl_out, msl_out, spv_out)),
    allow(
        dead_code,
        reason = "shared helpers can be dead if none of the enabled backends need it"
    )
)]

pub mod zero_init;

use alloc::{borrow::Cow, boxed::Box};

use crate::{
    compact::{compact, KeepUnused},
    valid::{Capabilities, ModuleInfo, ValidationError, ValidationFlags, Validator},
    Module, WithSpan,
};

/// An error produced while running the passes a backend requested.
#[derive(Clone, Debug, thiserror::Error)]
pub enum PassError {
    #[error(transparent)]
    ZeroInit(#[from] zero_init::ZeroInitError),
    #[error("module failed revalidation after backend passes")]
    Validation(#[from] Box<WithSpan<ValidationError>>),
}

/// The passes a backend requests, and their options.
#[derive(Clone, Debug)]
pub(crate) struct Passes {
    pub zero_init: zero_init::Options,
}

/// Run `passes` on `module`, then compact and revalidate it.
///
/// `module` must be valid, `info` must be its validation result, and its
/// overrides must have been processed with
/// [`process_overrides`](super::pipeline_constants::process_overrides).
///
/// If no pass changes the module, this returns `module` and `info` unchanged.
pub(crate) fn run<'a>(
    module: &'a Module,
    info: &'a ModuleInfo,
    passes: &Passes,
) -> Result<(Cow<'a, Module>, Cow<'a, ModuleInfo>), PassError> {
    let mut new_module = module.clone();
    let changed = zero_init::run(&mut new_module, info, &passes.zero_init)?;
    if !changed {
        return Ok((Cow::Borrowed(module), Cow::Borrowed(info)));
    }

    compact(&mut new_module, KeepUnused::Yes);
    let new_info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate_resolved_overrides(&new_module)?;
    Ok((Cow::Owned(new_module), Cow::Owned(new_info)))
}
