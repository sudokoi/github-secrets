//! Error formatting utilities.
//!
//! This module provides helper functions for formatting error chains
//! into human-readable messages.

use anyhow::Error;

/// Format an error and its source chain into a detailed error message.
///
/// This function extracts the full error chain from an `anyhow::Error`,
/// joining all error messages with " → " to provide comprehensive error context.
///
/// # Arguments
///
/// * `error` - The error to format
///
/// # Returns
///
/// A string containing the formatted error chain
///
/// # Example
///
/// ```
/// use anyhow::anyhow;
/// use github_secrets::error::format_error_chain;
///
/// let err = anyhow::anyhow!("outer error")
///     .context("middle error")
///     .context("inner error");
/// let formatted = format_error_chain(&err);
/// // Returns: "inner error → middle error → outer error"
/// ```
pub fn format_error_chain(error: &Error) -> String {
    let mut error_chain = vec![format!("{}", error)];
    let mut current = error.source();
    while let Some(err) = current {
        error_chain.push(format!("{}", err));
        current = err.source();
    }
    error_chain.join(" → ")
}
