use crate::GeneratorOptions;
use crate::diagnostic::Diagnostics;

/// Validate resolved generator options and emit warnings for conflicting settings.
///
/// With fixed SGC defaults, options can't conflict — this is a no-op.
/// Kept as a hook for future validation needs.
pub(crate) fn validate_options(_options: &GeneratorOptions, _diagnostics: &mut Diagnostics) {}
