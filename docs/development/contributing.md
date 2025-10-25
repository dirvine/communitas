# Contributing to Communitas

Thank you for your interest in contributing to Communitas! This guide will help you get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contribution Workflow](#contribution-workflow)
- [Pull Request Process](#pull-request-process)
- [Code Review](#code-review)
- [Community Guidelines](#community-guidelines)
- [Getting Help](#getting-help)

---

## Getting Started

### Ways to Contribute

**Code Contributions**:
- Bug fixes
- New features
- Performance improvements
- Test coverage improvements
- Documentation improvements

**Non-Code Contributions**:
- Bug reports
- Feature requests
- Documentation improvements
- Design feedback
- Community support

### Before You Start

1. **Check existing issues**: Look for existing issues or discussions
2. **Create an issue**: For significant changes, create an issue first
3. **Discuss approach**: Get feedback on your proposed solution
4. **Fork the repository**: Create your own fork to work on

---

## Development Setup

### Prerequisites

**Required Tools**:
- **Rust**: 1.85+ (stable channel)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Node.js**: 20+ with npm
  ```bash
  # Using nvm (recommended)
  nvm install 20
  nvm use 20
  ```

- **Git**: Latest stable version
  ```bash
  git --version  # Should be 2.30+
  ```

**Platform-Specific**:

**macOS**:
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows**:
- Install Visual Studio 2022 Build Tools
- Install Windows SDK

**Linux** (Ubuntu/Debian):
```bash
sudo apt update
sudo apt install build-essential libwebkit2gtk-4.1-dev libssl-dev
```

### Clone and Setup

```bash
# Fork the repository on GitHub first, then:
git clone https://github.com/YOUR_USERNAME/communitas.git
cd communitas

# Add upstream remote
git remote add upstream https://github.com/saorsalabs/communitas.git

# Install dependencies
npm install

# Build frontend
npm run build

# Verify setup
cargo test --all
npm test
```

### IDE Configuration

**Visual Studio Code** (Recommended):

**Required Extensions**:
- rust-analyzer
- Tauri
- ESLint
- Prettier

**Recommended Extensions**:
- Error Lens
- GitLens
- Better Comments
- Todo Tree

**settings.json**:
```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-features", "--", "-D", "warnings"],
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  }
}
```

---

## Contribution Workflow

### 1. Choose an Issue

**Good First Issues**:
- Look for issues labeled `good-first-issue`
- These are well-defined and suitable for newcomers
- Example: Documentation updates, simple bug fixes

**Claiming an Issue**:
```
Comment on the issue: "I'd like to work on this"
Wait for maintainer approval before starting
```

### 2. Create a Branch

```bash
# Update your fork
git checkout main
git fetch upstream
git merge upstream/main
git push origin main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/bug-description
```

**Branch Naming**:
- Features: `feature/short-description`
- Bug fixes: `fix/bug-description`
- Documentation: `docs/what-changed`
- Refactoring: `refactor/component-name`

### 3. Make Changes

**Code Quality Checklist**:
- [ ] Follow [coding standards](coding-standards.md)
- [ ] Add tests for new functionality
- [ ] Update documentation
- [ ] Run linters and formatters
- [ ] Verify all tests pass

**Before Committing**:
```bash
# Format code
cargo fmt --all
npm run prettier

# Run linters
cargo clippy --all-features -- -D warnings
npm run lint

# Run tests
cargo test --all
npm test

# Check types
npm run typecheck
```

### 4. Commit Your Changes

**Commit Message Format**:
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

**Good Commit Messages**:
```
feat(auth): add passkey authentication

Implement WebAuthn-based passkey authentication:
- Add passkey registration flow
- Implement Touch ID support for macOS
- Update authentication context
- Add comprehensive tests

Closes #123
```

```
fix(network): resolve connection timeout

The QUIC connection wasn't properly timing out, causing
indefinite hangs when peers were unreachable.

- Set timeout in QUIC configuration
- Add exponential backoff retry logic
- Improve error messages

Fixes #456
```

**Atomic Commits**:
- One logical change per commit
- Commit messages explain the "why"
- Each commit should build and pass tests

### 5. Push and Create Pull Request

```bash
# Push your branch
git push origin feature/your-feature-name

# Create pull request
gh pr create --base develop --title "feat: Your feature title" \
  --body "Description of your changes

## Changes
- Change 1
- Change 2

## Testing
- Describe how you tested

Closes #issue-number"
```

---

## Pull Request Process

### PR Template

When creating a PR, include:

```markdown
## Description
Brief description of changes and motivation.

## Changes
- Bullet list of specific changes
- Include both code and docs updates

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests pass locally

## Checklist
- [ ] Code follows coding standards
- [ ] Documentation updated
- [ ] Tests added for new functionality
- [ ] All tests passing
- [ ] No linting errors
- [ ] Commit messages follow convention

## Related Issues
Closes #123
Related to #456
```

### PR Quality Standards

**Required for All PRs**:
1. **Passes CI/CD**: All automated checks must pass
2. **Code Review**: At least one approval from maintainer
3. **Documentation**: Updated for any API changes
4. **Tests**: Comprehensive test coverage
5. **No Conflicts**: Rebased on latest develop branch

**CI/CD Checks**:
- ✅ Rust compilation (zero warnings)
- ✅ Clippy lints (strict mode)
- ✅ Code formatting
- ✅ All tests pass
- ✅ TypeScript type checking
- ✅ ESLint checks
- ✅ Security audit

### PR Size Guidelines

**Small PRs Preferred**:
- **Ideal**: < 200 lines changed
- **Acceptable**: 200-500 lines changed
- **Large**: 500-1000 lines (requires justification)
- **Too Large**: > 1000 lines (should be split)

**For Large Changes**:
1. Create an RFC (Request for Comments) issue first
2. Break into smaller, reviewable PRs
3. Use feature flags for incremental rollout
4. Document the overall architecture

### Updating Your PR

**Addressing Review Comments**:
```bash
# Make requested changes
# ... edit files ...

# Commit changes
git add -A
git commit -m "Address review comments

- Fix issue 1
- Update documentation
- Add requested tests"

# Push to update PR
git push origin feature/your-feature-name
```

**Rebasing on Develop**:
```bash
# Fetch latest changes
git fetch upstream

# Rebase your branch
git rebase upstream/develop

# Resolve any conflicts
# ... resolve conflicts ...
git add -A
git rebase --continue

# Force push (required after rebase)
git push origin feature/your-feature-name --force-with-lease
```

---

## Code Review

### Review Process

**Timeline**:
- **Initial Review**: Within 2 business days
- **Follow-up**: Within 1 business day
- **Final Approval**: After all comments addressed

**Review Criteria**:
1. **Correctness**: Does it work as intended?
2. **Code Quality**: Follows coding standards?
3. **Tests**: Adequate test coverage?
4. **Documentation**: Properly documented?
5. **Performance**: No obvious performance issues?
6. **Security**: No security vulnerabilities?

### Review Comments

**Types of Comments**:
- 🔴 **Must Fix**: Blocking issue that must be addressed
- 🟡 **Suggestion**: Recommended improvement
- 💡 **Nit**: Minor stylistic suggestion
- ❓ **Question**: Clarification needed
- ✅ **Approval**: Looks good!

**Responding to Comments**:
```
Good Response:
"Fixed in latest commit. Changed X to Y because of reason Z."

"Good point! I've updated the approach to use pattern A instead."

"I kept the original approach because [technical reason].
Happy to discuss alternatives if you have concerns."
```

### Being a Good Reviewer

**Constructive Feedback**:
```
❌ Bad: "This is wrong."
✅ Good: "Consider using pattern X here because Y.
Example: [code snippet]"

❌ Bad: "Why did you do it this way?"
✅ Good: "Could we use approach X instead?
It might be clearer because Y."
```

**Focus on**:
- Correctness and functionality
- Code quality and maintainability
- Test coverage
- Documentation
- Performance and security

**Avoid**:
- Personal preferences without technical merit
- Bikeshedding (debating trivial details)
- Blocking on style issues (let formatters handle it)

---

## Community Guidelines

### Code of Conduct

**Our Pledge**:
We are committed to providing a welcoming and inclusive community for everyone, regardless of:
- Experience level
- Gender identity and expression
- Sexual orientation
- Disability
- Personal appearance
- Race, ethnicity, or nationality
- Age
- Religion

**Our Standards**:

**Positive Behavior** ✅:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable Behavior** ❌:
- Trolling, insulting/derogatory comments, and personal attacks
- Public or private harassment
- Publishing others' private information
- Other conduct which could reasonably be considered inappropriate

### Communication Channels

**GitHub Issues**:
- Bug reports
- Feature requests
- Technical discussions

**Pull Requests**:
- Code review
- Implementation discussions

**Discord**:
- Real-time chat
- Community support
- General discussions

### Recognition

**Contributors**:
All contributors are recognized in:
- CONTRIBUTORS.md
- Release notes
- Project documentation

**Special Recognition**:
- Significant features
- Major improvements
- Consistent high-quality contributions
- Community leadership

---

## Getting Help

### Resources

**Documentation**:
- [Development Guide](README.md)
- [Coding Standards](coding-standards.md)
- [API Reference](../api/README.md)
- [Architecture Docs](../architecture/README.md)

**Getting Unstuck**:

**1. Check Documentation**:
- Look through guides and API docs
- Search existing issues and PRs

**2. Search Issues**:
```
site:github.com/saorsalabs/communitas your search terms
```

**3. Ask Questions**:
- Create a GitHub discussion
- Comment on relevant issues
- Join Discord (when available)

**4. Good Questions Include**:
- What you're trying to accomplish
- What you've already tried
- Relevant code snippets
- Error messages (full stack trace)
- Your environment (OS, Rust version, etc.)

### Common Questions

**Q: How do I run the app in development mode?**
```bash
npm run tauri dev
```

**Q: Tests are failing after I made changes**
```bash
# Run specific test
cargo test test_name -- --nocapture

# See full error output
RUST_LOG=debug cargo test

# Frontend tests
npm run test:ui  # Interactive mode
```

**Q: My PR has conflicts with develop**
```bash
git fetch upstream
git rebase upstream/develop
# Resolve conflicts
git push origin feature/branch --force-with-lease
```

**Q: How do I format my code?**
```bash
cargo fmt --all
npm run prettier
```

**Q: Where do I add my documentation?**
- API changes: Update `docs/api/`
- Features: Update relevant guide in `docs/guides/`
- Architecture: Update `docs/architecture/`

---

## Release Process

### Version Numbers

We use Semantic Versioning (SemVer):
- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features (backward compatible)
- **Patch** (0.0.1): Bug fixes

### Release Cycle

**Schedule**:
- **Major**: Annual (planned)
- **Minor**: Monthly
- **Patch**: As needed for critical bugs

**Process**:
1. Feature freeze (1 week before)
2. Release candidate testing
3. Final release
4. Changelog and release notes
5. Announcement

---

## Legal

### License

Communitas is licensed under GPL v3. By contributing, you agree that your contributions will be licensed under the same license.

### Contributor License Agreement

For significant contributions, we may ask you to sign a CLA (Contributor License Agreement) to ensure:
- We can use your contribution
- You have the right to contribute the code
- The project remains open source

---

## Thank You!

Thank you for contributing to Communitas! Every contribution, no matter how small, makes a difference. We appreciate your time and effort in making this project better.

**Happy Contributing! 🎉**

---

## See Also

- [Development Guide](README.md) - Complete development setup
- [Coding Standards](coding-standards.md) - Code quality guidelines
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
- [API Reference](../api/README.md) - API documentation

---

**Contributing Guide**: Together we build better software. 🤝💙
