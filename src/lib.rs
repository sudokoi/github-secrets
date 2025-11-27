//! # GitHub Secrets CLI
//!
//! A command-line tool for managing GitHub repository secrets interactively.
//!
//! This library provides functionality to:
//! - Load and validate configuration files
//! - Interact with GitHub's Actions Secrets API
//! - Provide an interactive TUI for secret management
//! - Validate inputs and handle errors gracefully
//!
//! ## Modules
//!
//! - [`config`] - Configuration file parsing and validation
//! - [`github`] - GitHub API client for secrets management
//! - [`prompt`] - Interactive terminal user interface
//! - [`validation`] - Input validation utilities
//! - [`paths`] - XDG-compliant path resolution
//! - [`error`] - Error formatting utilities
//! - [`errors`] - Structured error types
//! - [`constants`] - Application constants

pub mod app;
pub mod app_deps;
pub mod config;
pub mod constants;
pub mod error;
pub mod errors;
pub mod github;
pub mod paths;
pub mod prompt;
pub mod rate_limit;
pub mod validation;
