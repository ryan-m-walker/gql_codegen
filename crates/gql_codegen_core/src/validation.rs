use crate::PluginOptions;
use crate::diagnostic::Diagnostics;

/// Validate resolved plugin options and emit warnings for conflicting settings.
///
/// With fixed SGC defaults, options can't conflict — this is a no-op.
/// Kept as a hook for future validation needs.
pub(crate) fn validate_options(_options: &PluginOptions, _diagnostics: &mut Diagnostics) {}
