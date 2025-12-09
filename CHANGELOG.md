# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `config` subcommand for interactive configuration management
- Modal TUI dashboard for adding, editing, and deleting repositories
- Strict alphanumeric validation for repository owner and name fields

### Changed

- Updated key bindings for configuration dashboard (`e` for edit, `Enter` for edit/save)
- Refactored configuration management to use state-based UI logic
## [0.2.0] - 2025-11-21

### Added

- Comprehensive test coverage (63 tests total)
- Automatic changelog generation with git-cliff
- Integration tests for GitHub API interactions
- TUI interaction tests
- Test coverage reporting with Codecov
- Windows test isolation fixes
- Improved CI/CD pipeline with coverage reporting

### Changed

- Enhanced TUI with ratatui library
- Improved secret input UX with single input field
- Better error messages and validation feedback
- Codebase improvements and refactoring

## [0.1.0] - 2025-11-21

### Added

- Initial release
- Multi-repository secret management
- Interactive TUI for repository and secret selection
- XDG Config Directory support
- Rate limiting for GitHub API calls
- Input validation for secrets, repositories, and tokens
- Error handling with detailed error messages
- Retry functionality for failed operations
- CI/CD pipeline with automated testing

[Unreleased]: https://github.com/sudokoi/github-secrets/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/sudokoi/github-secrets/releases/tag/v0.2.0
[0.1.0]: https://github.com/sudokoi/github-secrets/releases/tag/v0.1.0
