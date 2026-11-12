//! Diagnostics: stable rule IDs, locations and fixes.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    Io,
    Json,
    LayerMetadata,
    Contract,
    Structure,
    Name,
    Description,
    Type,
    TypeUndetermined,
    ReferenceSyntax,
    ReferenceExternal,
    ReferenceMissing,
    ReferenceCycle,
    ReferenceType,
    Extends,
    Value,
    Namespace,
    Hierarchy,
    ComponentBinding,
    RequiredRole,
    ModeRoles,
    OverridePath,
    OverrideNamespace,
    OverrideTokenSet,
    OverrideType,
    OverridePrimitive,
    RoleConstraint,
    Layout,
    Typography,
    Usage,
    Contrast,
    SyntaxDistance,
    DesignSeed,
    FontAsset,
    IconAsset,
    CssName,
    OutputProtected,
    Deprecated,
}

impl Rule {
    /// Kebab-case ID of the variant, e.g. `reference-cycle`.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Rule::Io => "io",
            Rule::Json => "json",
            Rule::LayerMetadata => "layer-metadata",
            Rule::Contract => "contract",
            Rule::Structure => "structure",
            Rule::Name => "name",
            Rule::Description => "description",
            Rule::Type => "type",
            Rule::TypeUndetermined => "type-undetermined",
            Rule::ReferenceSyntax => "reference-syntax",
            Rule::ReferenceExternal => "reference-external",
            Rule::ReferenceMissing => "reference-missing",
            Rule::ReferenceCycle => "reference-cycle",
            Rule::ReferenceType => "reference-type",
            Rule::Extends => "extends",
            Rule::Value => "value",
            Rule::Namespace => "namespace",
            Rule::Hierarchy => "hierarchy",
            Rule::ComponentBinding => "component-binding",
            Rule::RequiredRole => "required-role",
            Rule::ModeRoles => "mode-roles",
            Rule::OverridePath => "override-path",
            Rule::OverrideNamespace => "override-namespace",
            Rule::OverrideTokenSet => "override-token-set",
            Rule::OverrideType => "override-type",
            Rule::OverridePrimitive => "override-primitive",
            Rule::RoleConstraint => "role-constraint",
            Rule::Layout => "layout",
            Rule::Typography => "typography",
            Rule::Usage => "usage",
            Rule::Contrast => "contrast",
            Rule::SyntaxDistance => "syntax-distance",
            Rule::DesignSeed => "design-seed",
            Rule::FontAsset => "font-asset",
            Rule::IconAsset => "icon-asset",
            Rule::CssName => "css-name",
            Rule::OutputProtected => "output-protected",
            Rule::Deprecated => "deprecated",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub rule: Rule,
    pub severity: Severity,
    /// `"night"` or `"night · fern"`.
    pub context: Option<String>,
    /// Root-relative path, optionally followed by `: <token path | #pointer>`.
    pub location: String,
    pub message: String,
    pub help: Option<String>,
}

impl Diagnostic {
    #[must_use]
    pub fn error(rule: Rule, location: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rule,
            severity: Severity::Error,
            context: None,
            location: location.into(),
            message: message.into(),
            help: None,
        }
    }

    #[must_use]
    pub fn warning(rule: Rule, location: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(rule, location, message)
        }
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN",
        };
        write!(f, "{level} [{}]", self.rule.id())?;
        if let Some(context) = &self.context {
            write!(f, " [{context}]")?;
        }
        write!(f, " {}: {}", self.location, self.message)?;
        if let Some(help) = &self.help {
            write!(f, "\n  help: {help}")?;
        }
        Ok(())
    }
}

/// A non-empty list of diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure(pub Vec<Diagnostic>);

impl Failure {
    #[must_use]
    pub fn one(d: Diagnostic) -> Self {
        Self(vec![d])
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, d) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{d}")?;
        }
        Ok(())
    }
}

pub type Outcome<T> = Result<T, Failure>;

/// Turn collected diagnostics into an outcome: `Err` when any is an error.
///
/// # Errors
/// Returns the diagnostics when at least one is an error.
pub fn finish<T>(value: T, diags: Vec<Diagnostic>) -> Outcome<T> {
    if diags.iter().any(|d| d.severity == Severity::Error) {
        Err(Failure(diags))
    } else {
        Ok(value)
    }
}
