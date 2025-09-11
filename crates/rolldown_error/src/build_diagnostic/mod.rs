pub mod constructors;
pub mod diagnostic;
pub mod events;

use std::{
  fmt::Display,
  ops::{Deref, DerefMut},
};

use crate::types::diagnostic_options::DiagnosticOptions;

use self::{diagnostic::Diagnostic, events::BuildEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
  Error,
  Warning,
}

#[derive(Debug)]
pub struct BuildDiagnostic {
  inner: Box<dyn BuildEvent>,
  severity: Severity,
  #[cfg(feature = "napi")]
  napi_error: Option<napi::Error>,
}

impl Display for BuildDiagnostic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.inner.message(&DiagnosticOptions::default()).fmt(f)
  }
}

impl BuildDiagnostic {
  fn new_inner(inner: impl Into<Box<dyn BuildEvent>>) -> Self {
    Self {
      inner: inner.into(),
      severity: Severity::Error,
      #[cfg(feature = "napi")]
      napi_error: None,
    }
  }

  pub fn id(&self) -> Option<String> {
    self.inner.id()
  }

  pub fn kind(&self) -> crate::types::event_kind::EventKind {
    self.inner.kind()
  }

  pub fn exporter(&self) -> Option<String> {
    self.inner.exporter()
  }

  pub fn severity(&self) -> Severity {
    self.severity
  }

  #[must_use]
  pub fn with_severity_warning(mut self) -> Self {
    self.severity = Severity::Warning;
    self
  }

  pub fn to_diagnostic(&self) -> Diagnostic {
    self.to_diagnostic_with(&DiagnosticOptions::default())
  }

  pub fn to_diagnostic_with(&self, opts: &DiagnosticOptions) -> Diagnostic {
    let mut diagnostic =
      Diagnostic::new(self.kind().to_string(), self.inner.message(opts), self.severity);
    self.inner.on_diagnostic(&mut diagnostic, opts);
    diagnostic
  }

  #[cfg(feature = "napi")]
  pub fn downcast_napi_error(&self) -> Result<&napi::Error, &Self> {
    match &self.napi_error {
      Some(napi_error) => Ok(napi_error),
      None => Err(self),
    }
  }
}

#[cfg(feature = "napi")]
impl From<napi::Error> for BuildDiagnostic {
  fn from(e: napi::Error) -> Self {
    BuildDiagnostic::napi_error(e)
  }
}

impl From<BuildDiagnostic> for BatchedBuildDiagnostic {
  fn from(v: BuildDiagnostic) -> Self {
    Self::new(vec![v])
  }
}

impl From<anyhow::Error> for BatchedBuildDiagnostic {
  fn from(err: anyhow::Error) -> Self {
    Self::new(vec![BuildDiagnostic::unhandleable_error(err)])
  }
}

impl From<Vec<BuildDiagnostic>> for BatchedBuildDiagnostic {
  fn from(v: Vec<BuildDiagnostic>) -> Self {
    Self::new(v)
  }
}

impl From<anyhow::Error> for BuildDiagnostic {
  fn from(err: anyhow::Error) -> Self {
    BuildDiagnostic::unhandleable_error(err)
  }
}

#[derive(Debug, Default)]
pub struct BatchedBuildDiagnostic(Vec<BuildDiagnostic>);

impl BatchedBuildDiagnostic {
  pub fn new(vec: Vec<BuildDiagnostic>) -> Self {
    Self(vec)
  }

  pub fn into_vec(self) -> Vec<BuildDiagnostic> {
    self.0
  }
}

impl Deref for BatchedBuildDiagnostic {
  type Target = Vec<BuildDiagnostic>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for BatchedBuildDiagnostic {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
