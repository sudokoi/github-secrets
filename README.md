# github-secrets

A command-line tool for securely updating GitHub repository secrets across multiple repositories. Built with Rust for performance and security.

## Features

- **Multi-repository support**: Update secrets across multiple repositories in a single run
- **Interactive selection**: Choose specific repositories or select all at once
- **Secure encryption**: Uses NaCl box encryption (pure Rust, no system dependencies)
- **Confirmation prompts**: Shows last update date and asks for confirmation before overwriting existing secrets
- **Error handling**: Continues processing on errors and provides retry functionality
- **Detailed summaries**: Per-repository breakdown and overall operation statistics

## Installation

### Prerequisites

- Rust 1.91.1 or later
- A GitHub Personal Access Token with `repo` (for private repos) or `public_repo` (for public repos) permissions

### Build from Source

```bash
git clone <repository-url>
cd github-secrets
cargo build --release
```

The binary will be available at `target/release/github-secrets`.

## Usage

### Initial Setup

1. **Create a `.env` file** in the project root:

```env
GITHUB_TOKEN=your_github_token_here
```

2. **Create a `config.toml` file** (copy from `config.example.toml`):

```toml
[[repositories]]
owner = "your_github_username"
name = "your_repository_name"
alias = "My Main Repo"  # Optional friendly name

[[repositories]]
owner = "your_github_username"
name = "another_repository"
alias = "Secondary Repo"  # Optional friendly name
```

### Running the Tool

```bash
cargo run
# or if installed:
github-secrets
```

### Workflow

1. **Select repositories**: Choose one or more repositories from the interactive menu, or select "Select All" to update all repositories
2. **Enter secrets**: Input key-value pairs interactively
   - Press `ESC` to finish entering secrets (requires confirmation)
   - Empty keys or values are skipped
3. **Confirm overwrites**: If a secret already exists, you'll be shown the last update date and asked for confirmation
4. **Review summary**: See overall statistics and per-repository breakdown
5. **Retry failed operations**: Option to retry any failed secret updates

### Environment Variables

- `GITHUB_TOKEN`: Required. Your GitHub Personal Access Token
- `CONFIG_PATH`: Optional. Path to config file (defaults to `config.toml`)

### Example Session

```
Select repositories to update secrets for:
> [x] Select All
  [ ] My Main Repo (user/repo1)
  [ ] Secondary Repo (user/repo2)

Selected all 2 repositories:
  - My Main Repo (user/repo1)
  - Secondary Repo (user/repo2)

Enter secret key-value pairs. Press ESC to finish.
Secret key (or ESC to finish): API_KEY
Secret value: sk_live_abc123...
Secret pair added.

Secret key (or ESC to finish): [ESC]
Are you sure you want to exit? (y/N): y

Processing 1 secret(s) across 2 repository/repositories...

============================================================
Repository: My Main Repo (user/repo1)
============================================================
✓ Successfully updated secret 'API_KEY' in My Main Repo (user/repo1)

============================================================
Repository: Secondary Repo (user/repo2)
============================================================
✓ Successfully updated secret 'API_KEY' in Secondary Repo (user/repo2)

============================================================
Overall Summary
============================================================
Total operations: 2
Successful: 2
Failed: 0

Per-repository breakdown:
  My Main Repo (user/repo1): 1 successful, 0 failed
  Secondary Repo (user/repo2): 1 successful, 0 failed
```

## Local Development Setup

### Requirements

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Git

### Setup Steps

1. **Clone the repository**:

```bash
git clone <repository-url>
cd github-secrets
```

2. **Install dependencies**:

```bash
cargo build
```

3. **Set up environment**:

```bash
cp .env.example .env
# Edit .env and add your GITHUB_TOKEN
```

4. **Create config file**:

```bash
cp config.example.toml config.toml
# Edit config.toml with your repository details
```

5. **Run the tool**:

```bash
cargo run
```

### Running Tests

```bash
cargo test
```

### Building for Release

```bash
cargo build --release
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Project Structure

```
github-secrets/
├── src/
│   ├── main.rs          # Entry point and orchestration
│   ├── config.rs        # Configuration file parsing
│   ├── github.rs        # GitHub API client and encryption
│   └── prompt.rs        # Interactive user prompts
├── Cargo.toml           # Project dependencies
├── config.example.toml  # Example configuration
├── .env.example         # Example environment variables
└── README.md            # This file
```

## Security

- Secrets are encrypted using NaCl box encryption before being sent to GitHub
- All encryption is done in pure Rust (no system dependencies)
- GitHub token is read from environment variables, never hardcoded
- Secret values are never logged or displayed after input

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Author

**Sudhanshu Ranjan**

- GitHub: [@sudokoi](https://github.com/sudokoi)
- Email: perfectsudh@gmail.com
