# Pull Request Guidelines

Comprehensive guidelines for submitting and reviewing pull requests in Cobalt Stack.

## Table of Contents

- [Before You Start](#before-you-start)
- [Creating a Pull Request](#creating-a-pull-request)
- [Pull Request Template](#pull-request-template)
- [PR Checklist](#pr-checklist)
- [Review Process](#review-process)
- [Merging Strategy](#merging-strategy)
- [Common Scenarios](#common-scenarios)
- [Troubleshooting](#troubleshooting)

## Before You Start

### Prerequisites

Before creating a pull request:

1. **Read the Guidelines**
   - [CONTRIBUTING.md](../../CONTRIBUTING.md)
   - [Code Style Guide](./code-style.md)
   - [Testing Requirements](./testing-requirements.md)

2. **Verify Your Changes**
   - Code follows style guidelines
   - All tests pass locally
   - No linting errors
   - Documentation is updated

3. **Update Your Branch**
   - Branch is up-to-date with main
   - Conflicts are resolved
   - Commits are clean

### Branch Naming

Use descriptive branch names following these patterns:

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feature/description` | `feature/email-verification` |
| Bug Fix | `fix/description` | `fix/login-redirect-issue` |
| Documentation | `docs/description` | `docs/api-reference` |
| Refactoring | `refactor/description` | `refactor/auth-service` |
| Testing | `test/description` | `test/integration-tests` |
| Chore | `chore/description` | `chore/update-dependencies` |

## Creating a Pull Request

### Step-by-Step Process

#### 1. Ensure Your Branch is Up-to-Date

```bash
# Fetch latest changes
git fetch upstream

# Rebase your branch
git checkout feature/your-feature
git rebase upstream/main

# If there are conflicts, resolve them
git add .
git rebase --continue

# Force push (if needed after rebase)
git push origin feature/your-feature --force-with-lease
```

#### 2. Run Final Checks

**Backend (Rust)**:
```bash
cd backend

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests
cargo test

# Check build
cargo build
```

**Frontend (TypeScript)**:
```bash
cd frontend

# Lint code
npm run lint

# Format code
npm run format

# Type check
npm run type-check

# Run tests
npm test

# Build
npm run build
```

#### 3. Push Your Changes

```bash
git push origin feature/your-feature
```

#### 4. Open Pull Request

1. Go to the repository on GitHub
2. Click "New Pull Request"
3. Select your branch
4. Fill out the PR template (see below)
5. Request reviewers
6. Add labels
7. Submit the PR

## Pull Request Template

Use this template when creating a pull request:

```markdown
## Description

[Provide a clear description of your changes]

### Problem

[What problem does this solve? Link to related issues]

### Solution

[How does this PR solve the problem?]

## Related Issues

Fixes #[issue number]
Closes #[issue number]
Related to #[issue number]

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test improvement

## Testing

### How Has This Been Tested?

[Describe the tests you ran to verify your changes]

- [ ] Unit tests
- [ ] Integration tests
- [ ] E2E tests
- [ ] Manual testing

### Test Configuration

- **Rust Version**: 1.70
- **Node Version**: 18.x
- **PostgreSQL Version**: 14
- **OS**: Linux/macOS/Windows

## Screenshots (if applicable)

[Add screenshots to demonstrate UI changes]

## Checklist

### Code Quality

- [ ] My code follows the code style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings
- [ ] I have removed debug/console logs

### Testing

- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have checked test coverage
- [ ] I have tested error scenarios

### Documentation

- [ ] I have updated the documentation accordingly
- [ ] I have updated code comments
- [ ] I have updated the API documentation (if applicable)
- [ ] I have updated the CHANGELOG (if applicable)

### Dependencies

- [ ] I have not introduced new dependencies without discussion
- [ ] I have updated dependency versions if needed
- [ ] I have checked for security vulnerabilities in dependencies

## Additional Context

[Add any other context about the PR here]

## Breaking Changes

[If this is a breaking change, describe what breaks and how to migrate]

## Deployment Notes

[Any special deployment considerations]
```

## PR Checklist

### Before Submitting

- [ ] **Code Quality**
  - Follows code style guidelines
  - No linting errors
  - Proper formatting (rustfmt/prettier)
  - Self-review completed

- [ ] **Testing**
  - All tests pass
  - New tests added for new functionality
  - Edge cases covered
  - Test coverage maintained or improved

- [ ] **Documentation**
  - Code comments updated
  - API documentation updated
  - User guides updated (if applicable)
  - CHANGELOG updated (if applicable)

- [ ] **Git Hygiene**
  - Branch is up-to-date with main
  - Commit messages are clear
  - No merge commits (use rebase)
  - Commits are atomic and logical

### During Review

- [ ] Respond to review comments promptly
- [ ] Make requested changes
- [ ] Mark conversations as resolved
- [ ] Keep PR updated with main branch

### Before Merging

- [ ] All review comments addressed
- [ ] All CI checks passing
- [ ] At least one approval from maintainer
- [ ] Documentation is complete
- [ ] No merge conflicts

## Review Process

### Timeline

| Stage | Expected Time |
|-------|---------------|
| Initial Review | 1-2 business days |
| Follow-up Reviews | 1 business day |
| Approval to Merge | Same day |

*Note: Timeline may vary based on PR complexity and team availability*

### Review Criteria

Reviewers evaluate PRs based on:

#### 1. Correctness
- Does the code work as intended?
- Are there any bugs or edge cases?
- Are error cases handled properly?

#### 2. Code Quality
- Follows coding standards?
- Is the code readable and maintainable?
- Are there any code smells?
- Is there unnecessary complexity?

#### 3. Testing
- Are there adequate tests?
- Do tests cover edge cases?
- Are tests well-structured?

#### 4. Performance
- Are there any performance concerns?
- Are database queries optimized?
- Are there unnecessary allocations?

#### 5. Security
- Are there any security vulnerabilities?
- Is input validation adequate?
- Are secrets properly handled?

#### 6. Documentation
- Is the code well-documented?
- Is the API documentation updated?
- Are user guides updated?

### Requesting Changes

Reviewers may:

- **Comment**: Questions or suggestions
- **Request Changes**: Issues that must be addressed
- **Approve**: PR is ready to merge

### Responding to Reviews

#### Good Practices

- Respond to all comments
- Ask for clarification if needed
- Explain your reasoning
- Be open to feedback
- Thank reviewers for their time

#### Example Responses

**Accepting Feedback**:
```
Good catch! I'll update the error handling to include that case.
```

**Requesting Clarification**:
```
Could you elaborate on what you mean by "simplify this logic"?
I want to make sure I understand your suggestion correctly.
```

**Explaining Decision**:
```
I chose this approach because [reason]. However, I'm open to
alternatives if you think there's a better way.
```

**Disagreeing Constructively**:
```
I understand your concern about performance, but I ran benchmarks
and the difference is negligible (see attached results). However,
if you still think we should optimize this, I'm happy to do so.
```

## Merging Strategy

### Merge Methods

We use **Squash and Merge** for most PRs:

- Keeps main branch history clean
- One commit per feature/fix
- Easier to revert if needed

### Before Merging

1. **Ensure CI Passes**
   - All tests pass
   - Linting succeeds
   - Build succeeds

2. **Update Branch**
   - Rebase on latest main
   - Resolve any conflicts

3. **Get Approval**
   - At least one approval from maintainer
   - All review comments resolved

4. **Final Review**
   - Quick self-review of all changes
   - Verify commit message

### Merge Commit Message

The squashed commit should follow this format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Example**:
```
feat(auth): add email verification

- Add email verification token generation
- Implement email sending service
- Create verification endpoint
- Add verification UI

Closes #123
```

## Common Scenarios

### Small Bug Fix

```markdown
## Description
Fix CORS configuration in production environment

## Related Issues
Fixes #456

## Type of Change
- [x] Bug fix

## Testing
- [x] Manual testing in production environment
```

### New Feature

```markdown
## Description
Add email verification for user registration

## Related Issues
Closes #123

## Type of Change
- [x] New feature

## Testing
- [x] Unit tests for token generation
- [x] Integration tests for verification endpoint
- [x] Manual testing of email flow
```

### Documentation Update

```markdown
## Description
Update API documentation with examples

## Type of Change
- [x] Documentation update

## Checklist
- [x] Documentation updated
- [x] Examples tested
- [x] Links verified
```

### Breaking Change

```markdown
## Description
Change authentication response format

## Type of Change
- [x] Breaking change

## Breaking Changes
The authentication response now returns `access_token` instead of `token`.

### Migration
```javascript
// Before
const { token } = await login();

// After
const { access_token } = await login();
```

## Deployment Notes
Update client applications to use new response format.
```

## Troubleshooting

### CI Failing

**Problem**: Tests fail in CI but pass locally

**Solution**:
1. Check CI logs for specific errors
2. Ensure all dependencies are installed
3. Verify environment variables
4. Run tests with same configuration as CI

### Merge Conflicts

**Problem**: Conflicts with main branch

**Solution**:
```bash
# Update your branch
git fetch upstream
git checkout feature/your-feature
git rebase upstream/main

# Resolve conflicts
# Edit conflicting files
git add .
git rebase --continue

# Push updated branch
git push origin feature/your-feature --force-with-lease
```

### Failed Linting

**Problem**: Linting errors in CI

**Solution**:
```bash
# Backend
cd backend
cargo fmt
cargo clippy --fix

# Frontend
cd frontend
npm run lint -- --fix
npm run format

# Commit and push
git add .
git commit -m "fix: resolve linting errors"
git push
```

### Large PR

**Problem**: PR is too large to review

**Solution**:
1. Break into smaller, focused PRs
2. Submit incrementally
3. Each PR should be independently reviewable

### Stale PR

**Problem**: PR is outdated and conflicts with main

**Solution**:
```bash
# Rebase on latest main
git fetch upstream
git rebase upstream/main

# Resolve conflicts if any
# Force push
git push origin feature/your-feature --force-with-lease
```

## Best Practices

### Do's

- Keep PRs focused and small
- Write clear descriptions
- Add tests for new features
- Update documentation
- Respond to reviews promptly
- Be respectful and professional

### Don'ts

- Don't submit PRs with failing tests
- Don't include unrelated changes
- Don't ignore review comments
- Don't force push without communication
- Don't merge your own PRs (wait for approval)

## Resources

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Code Style Guide](./code-style.md)
- [Testing Requirements](./testing-requirements.md)
- [GitHub PR Documentation](https://docs.github.com/en/pull-requests)

## Questions?

If you have questions about pull requests:

1. Check this documentation
2. Search GitHub issues
3. Ask in GitHub Discussions
4. Reach out to maintainers
