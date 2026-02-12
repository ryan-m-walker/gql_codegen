use crate::PluginOptions;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Diagnostics};

/// Validate resolved plugin options and emit warnings for conflicting settings.
pub(crate) fn validate_options(options: &PluginOptions, diagnostics: &mut Diagnostics) {
    let enums_as_types = options.enums_as_types.unwrap_or(true);

    if options.future_proof_enums && !enums_as_types {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCategory::Config,
            "`futureProofEnums` has no effect when `enumsAsTypes` is enabled",
        ));
    }

    if options.numeric_enums && enums_as_types {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCategory::Config,
            "`enumsAsTypes` has no effect when `numericEnums` is enabled",
        ));
    }
}
