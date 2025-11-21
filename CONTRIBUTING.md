# Contributing to github-secrets

Thank you for your interest in contributing to github-secrets! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and considerate of others
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Respect different viewpoints and experiences

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/github-secrets.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test your changes: `cargo test && cargo build`
6. Commit your changes: `git commit -m "Add feature: description"`
7. Push to your fork: `git push origin feature/your-feature-name`
8. Open a Pull Request

## Development Workflow

### Setting Up Development Environment

1. Ensure you have Rust installed (1.91.1 or later)
2. Clone the repository
3. Run `cargo build` to fetch dependencies
4. Copy `.env.example` to `.env` and add your GitHub token
5. Copy `config.example.toml` to `config.toml` and configure repositories

### Running Tests

```bash
cargo test
```

### Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy for linting: `cargo clippy`
- Ensure all tests pass before submitting

### Commit Messages

- Use imperative mood ("Add feature" not "Added feature")
- Keep the first line under 72 characters
- Provide context in the body if needed
- Reference issue numbers if applicable

Example:
```
Add support for environment variable secrets

This change allows users to reference environment variables
in the config file instead of hardcoding values.

Fixes #123
```

## Pull Request Process

1. **Update Documentation**: If you're adding features, update the README.md
2. **Add Tests**: Include tests for new functionality
3. **Ensure Tests Pass**: All existing and new tests should pass
4. **Update CHANGELOG**: Document your changes (if applicable)
5. **Request Review**: Assign reviewers and wait for feedback

### PR Checklist

- [ ] Code follows the project's style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] No new warnings introduced

## Areas for Contribution

### Bug Fixes

- Fix issues reported in the issue tracker
- Include tests that demonstrate the bug and verify the fix
- Reference the issue number in your PR

### New Features

- Discuss major features in an issue first
- Ensure the feature aligns with the project's goals
- Include comprehensive tests
- Update documentation

### Documentation

- Improve clarity of existing documentation
- Add examples for complex features
- Fix typos and grammatical errors
- Add code comments where needed

### Testing

- Add tests for uncovered code paths
- Improve test coverage
- Add integration tests for new features

## Code Review Guidelines

### For Contributors

- Be open to feedback and suggestions
- Respond to review comments promptly
- Make requested changes or explain why they might not be necessary
- Keep PRs focused and reasonably sized

### For Reviewers

- Be constructive and respectful
- Explain the reasoning behind suggestions
- Approve when satisfied, or request changes with clear guidance
- Test the changes locally when possible

## Reporting Issues

### Bug Reports

Include:
- Clear description of the bug
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version)
- Relevant error messages or logs

### Feature Requests

Include:
- Clear description of the feature
- Use cases and motivation
- Potential implementation approach (if you have ideas)

## Questions?

Feel free to:
- Open an issue for discussion
- Contact the maintainer: perfectsudh@gmail.com
- Check existing issues and discussions

Thank you for contributing to github-secrets!

