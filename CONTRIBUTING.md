# Contributing to Devilbox Boost

Thank you for considering contributing to Devilbox Boost! This document provides guidelines and information for contributors.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Pull Request Process](#pull-request-process)

---

## Code of Conduct

This project adheres to a simple code of conduct:

- **Be respectful** - Treat everyone with respect
- **Be constructive** - Provide helpful feedback  
- **Be collaborative** - Work together to improve the project
- **Be inclusive** - Welcome contributors of all skill levels

---

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

**Good bug reports** include:
- Clear, descriptive title
- Exact steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Docker version, PHP version)

### Suggesting Enhancements

Enhancement suggestions are welcome! Please include:
- Clear use case
- Why this would be useful
- How it should work

---

## Development Setup

### Prerequisites

- Docker Desktop installed and running
- Git
- Basic understanding of Docker, PHP, and Bash

### Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR-USERNAME/devilbox-boost.git`
3. Create a feature branch: `git checkout -b feature/my-new-feature`
4. Make your changes
5. Test thoroughly
6. Commit with clear messages
7. Push and create a Pull Request

---

## Coding Standards

### Shell Scripts (Bash)

```bash
#!/usr/bin/env bash
set -e  # Exit on error

# Use clear variable names
MY_VARIABLE="value"

# Add comments for complex logic
function my_function() {
    local param="$1"
    # Function logic
}
```

### PHP

Follow PSR-12 coding standards

### Documentation

- Use Markdown for all documentation
- Keep line length under 100 characters
- Include code examples where applicable

---

## Pull Request Process

### Before Submitting

- [ ] Code follows project standards
- [ ] Changes tested locally
- [ ] Documentation updated
- [ ] Commit messages are clear

### PR Guidelines

1. **Clear Title** - Use prefixes: `feat:`, `fix:`, `docs:`, `refactor:`
2. **Detailed Description** - What, why, and how
3. **Link Related Issues** - Use `Fixes #123`
4. **Small, Focused PRs** - One feature/fix per PR

---

## Questions?

If you have questions:
1. Check existing documentation
2. Search closed issues
3. Ask in GitHub Discussions
4. Open a new issue with the `question` label

---

## License

By contributing to Devilbox Boost, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing!** 🚀
