# Contributing to Cobalt Stack

Thank you for your interest in contributing to Cobalt Stack! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Code Review Guidelines](#code-review-guidelines)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Community](#community)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and professional in all interactions.

### Our Standards

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust**: 1.70 or higher
- **Node.js**: 18.x or higher
- **PostgreSQL**: 14 or higher
- **Valkey/Redis**: 7.x or higher
- **Git**: For version control

### Development Setup

1. **Fork the repository** on GitHub

2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/cobalt-stack.git
   cd cobalt-stack
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/ORIGINAL_OWNER/cobalt-stack.git
   ```

4. **Install dependencies**:
   ```bash
   # Backend
   cd backend
   cargo build

   # Frontend
   cd ../frontend
   npm install
   ```

5. **Set up environment**:
   ```bash
   # Copy example environment file
   cp .env.example .env

   # Edit .env with your configuration
   ```

6. **Run database migrations**:
   ```bash
   cd backend
   cargo run --bin migration
   ```

7. **Start development servers**:
   ```bash
   # Terminal 1: Backend
   cd backend
   cargo run

   # Terminal 2: Frontend
   cd frontend
   npm run dev
   ```

For detailed setup instructions, see [Installation Guide](docs/getting-started/installation.md).

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug Reports**: Report issues you encounter
- **Feature Requests**: Suggest new features or improvements
- **Code Contributions**: Fix bugs or implement features
- **Documentation**: Improve or add documentation
- **Testing**: Write or improve tests
- **Code Review**: Review pull requests

### Reporting Bugs

Before creating a bug report:

1. Check if the issue already exists
2. Ensure you're using the latest version
3. Collect relevant information

When reporting a bug, include:

- **Description**: Clear description of the issue
- **Steps to Reproduce**: Detailed steps to reproduce the behavior
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Rust version, Node version, etc.
- **Logs**: Relevant error messages or logs

### Suggesting Features

When suggesting a feature:

- **Use Case**: Explain why this feature is needed
- **Description**: Clear description of the proposed feature
- **Alternatives**: Any alternative solutions you've considered
- **Implementation Ideas**: If you have thoughts on implementation

## Development Workflow

### Branching Strategy

We use a feature branch workflow:

1. **Main Branch**: `main` (production-ready code)
2. **Feature Branches**: `feature/description`
3. **Bug Fix Branches**: `fix/description`
4. **Documentation Branches**: `docs/description`

### Creating a Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a new branch
git checkout -b feature/your-feature-name
```

### Making Changes

1. **Write Code**: Implement your changes
2. **Follow Standards**: Adhere to [coding standards](docs/contributing/code-style.md)
3. **Add Tests**: Include tests for new functionality
4. **Update Documentation**: Document your changes
5. **Commit Changes**: Make clear, atomic commits

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples**:
```bash
feat(auth): add email verification

- Add email verification token generation
- Implement email sending service
- Create verification endpoint

Closes #123

fix(api): resolve CORS issue in production

The CORS middleware was not properly configured for production
environment. This commit adds the correct allowed origins.

Fixes #456

docs(api): update authentication documentation

Add examples for token refresh endpoint
```

## Pull Request Process

### Before Submitting

Ensure your PR meets these requirements:

- [ ] Code follows [coding standards](docs/contributing/code-style.md)
- [ ] All tests pass locally
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] Commit messages are clear and follow conventions
- [ ] Branch is up-to-date with main

### Submitting a Pull Request

1. **Push your branch**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Open a Pull Request** on GitHub

3. **Fill out the PR template** with:
   - Description of changes
   - Related issues
   - Testing performed
   - Screenshots (if applicable)

4. **Request reviewers**

### Pull Request Template

```markdown
## Description
[Describe your changes]

## Related Issues
Fixes #[issue number]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
[Describe testing performed]

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review performed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] No new warnings
```

### After Submitting

- Respond to review comments
- Make requested changes
- Keep your branch updated with main
- Be patient and respectful

## Code Review Guidelines

### For Authors

- Be open to feedback
- Respond to comments promptly
- Ask questions if feedback is unclear
- Make requested changes or discuss alternatives

### For Reviewers

- Be respectful and constructive
- Focus on the code, not the person
- Explain why changes are needed
- Approve when satisfied with changes

### Review Criteria

Reviewers check for:

- **Correctness**: Does the code work as intended?
- **Testing**: Are there adequate tests?
- **Style**: Does it follow coding standards?
- **Performance**: Are there any performance concerns?
- **Security**: Are there any security issues?
- **Documentation**: Is it well-documented?

## Coding Standards

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write comprehensive documentation comments
- Keep functions focused and small

### TypeScript/JavaScript

- Follow the [Airbnb JavaScript Style Guide](https://github.com/airbnb/javascript)
- Use ESLint and Prettier
- Use TypeScript for type safety
- Write JSDoc comments for public APIs

### General Principles

- **DRY**: Don't Repeat Yourself
- **KISS**: Keep It Simple and Straightforward
- **SOLID**: Follow SOLID principles
- **Clean Code**: Write self-documenting code
- **Error Handling**: Handle errors gracefully

For detailed guidelines, see [Code Style Guide](docs/contributing/code-style.md).

## Testing Requirements

### Backend (Rust)

- **Unit Tests**: Test individual functions
- **Integration Tests**: Test API endpoints
- **Minimum Coverage**: 70% for new code

```bash
# Run tests
cargo test

# Run with coverage
cargo tarpaulin --out Html
```

### Frontend (TypeScript)

- **Unit Tests**: Test components and utilities
- **Integration Tests**: Test user flows
- **E2E Tests**: Test critical paths

```bash
# Run tests
npm test

# Run with coverage
npm run test:coverage
```

### Test Guidelines

- Write tests for new features
- Update tests for modified code
- Ensure tests are deterministic
- Use descriptive test names
- Mock external dependencies

For detailed requirements, see [Testing Requirements](docs/contributing/testing-requirements.md).

## Documentation

### Types of Documentation

- **Code Comments**: Explain complex logic
- **API Documentation**: Document public APIs
- **User Guides**: Help users understand features
- **Developer Guides**: Help contributors understand the codebase

### Documentation Standards

- Write clear, concise documentation
- Include examples where helpful
- Keep documentation up-to-date
- Use proper grammar and spelling

### Generating Documentation

**Rust**:
```bash
cargo doc --no-deps --open
```

**TypeScript**:
```bash
npm run docs
```

## Community

### Getting Help

- **GitHub Discussions**: Ask questions and discuss ideas
- **GitHub Issues**: Report bugs or request features
- **Pull Requests**: Contribute code and documentation

### Communication Guidelines

- Be respectful and professional
- Stay on topic
- Provide context for your questions
- Search before asking

### Recognition

Contributors are recognized in:

- [CHANGELOG.md](docs/CHANGELOG.md)
- Project documentation
- Release notes

## Additional Resources

- [Installation Guide](docs/getting-started/installation.md)
- [Quick Start Guide](docs/getting-started/quick-start.md)
- [Project Structure](docs/getting-started/project-structure.md)
- [API Documentation](docs/api/)
- [Code Style Guide](docs/contributing/code-style.md)
- [Testing Requirements](docs/contributing/testing-requirements.md)
- [Pull Request Guidelines](docs/contributing/pull-requests.md)

## Questions?

If you have questions about contributing:

1. Check existing documentation
2. Search GitHub issues and discussions
3. Ask in GitHub Discussions
4. Open a new issue if needed

Thank you for contributing to Cobalt Stack!
