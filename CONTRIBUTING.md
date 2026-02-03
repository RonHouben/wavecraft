# Contributing to Wavecraft

Thank you for your interest in contributing to Wavecraft! This document provides guidelines for contributing to the project.

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md). We are committed to providing a welcoming and inclusive environment for all contributors.

## Development Setup

### Prerequisites

- **Rust** (stable, 2024 edition)
- **Node.js** 20+
- **CMake** (for AU wrapper on macOS)

### Getting Started

1. **Clone the repository:**
   ```bash
   git clone https://github.com/RonHouben/wavecraft.git
   cd wavecraft
   ```

2. **Install dependencies:**
   ```bash
   cd ui && npm install
   ```

3. **Build and test:**
   ```bash
   cd engine
   cargo xtask test        # Run all tests
   cargo xtask lint        # Check code quality
   cargo xtask bundle      # Build plugin bundles
   ```

For detailed setup instructions, see the [SDK Getting Started Guide](docs/guides/sdk-getting-started.md).

## Coding Standards

Wavecraft follows strict coding standards to maintain code quality and consistency. Please read [docs/architecture/coding-standards.md](docs/architecture/coding-standards.md) before contributing.

**Key points:**

- **Rust:** Class-based architecture (non-React code), real-time safety in audio thread
- **TypeScript:** Functional components with hooks, strict mode enabled
- **Formatting:** Run `cargo xtask lint --fix` before committing
- **Testing:** Add tests for new functionality
- **Commit messages:** Use conventional commits format (feat:, fix:, docs:, etc.)

## Pull Request Process

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   - Follow the coding standards
   - Add tests for new functionality
   - Update documentation as needed

3. **Run quality checks:**
   ```bash
   cargo xtask lint --fix  # Fix linting issues
   cargo xtask test        # Run all tests
   ```

4. **Commit your changes:**
   ```bash
   git commit -m "feat: add your feature description"
   ```
   
   Use conventional commit prefixes:
   - `feat:` — New feature
   - `fix:` — Bug fix
   - `docs:` — Documentation changes
   - `refactor:` — Code refactoring
   - `test:` — Adding or updating tests
   - `chore:` — Maintenance tasks

5. **Push and create PR:**
   ```bash
   git push origin feature/your-feature-name
   ```
   
   Then open a pull request on GitHub using the provided template.

## Testing Requirements

All contributions must include appropriate tests:

- **UI changes:** Add unit tests using Vitest + React Testing Library
- **Engine changes:** Add Rust unit tests and integration tests
- **Run tests locally:** `cargo xtask test` before submitting PR

See [docs/architecture/coding-standards.md#testing](docs/architecture/coding-standards.md#testing) for testing guidelines.

## Documentation

Update documentation when:

- Adding new features or APIs
- Changing existing behavior
- Adding configuration options
- Fixing bugs that might affect users

Documentation lives in `docs/`:
- `docs/guides/` — User-facing guides
- `docs/architecture/` — Technical architecture documents
- `docs/feature-specs/` — Feature specifications (for development)

## CI/CD Pipeline

All PRs run through our CI pipeline:

1. **Linting** — ESLint + Prettier (UI), Clippy + fmt (Engine)
2. **Type checking** — TypeScript strict mode
3. **Tests** — UI unit tests, Engine tests
4. **Build** — VST3 + CLAP bundles (main branch only)

PRs must pass all CI checks before merging. See [docs/guides/ci-pipeline.md](docs/guides/ci-pipeline.md) for details.

## Squash Merge Policy

All PRs are squashed on merge to keep the main branch history clean. Your PR title becomes the commit message, so make it descriptive.

## Questions or Need Help?

- **Documentation:** Check [docs/](docs/) first
- **Discussions:** Use [GitHub Discussions](https://github.com/RonHouben/wavecraft/discussions)
- **Issues:** Report bugs or request features via [GitHub Issues](https://github.com/RonHouben/wavecraft/issues)

## License

By contributing to Wavecraft, you agree that your contributions will be licensed under the [MIT License](LICENSE).

---

Thank you for contributing to Wavecraft! 🎵
