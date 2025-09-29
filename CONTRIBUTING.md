# Contributing to Hyperliquid Trading Bot

Thank you for your interest in contributing to the Hyperliquid Trading Bot! This document provides guidelines and information for contributors.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ installed
- Git configured
- Basic understanding of trading concepts
- Familiarity with Rust (helpful but not required)

### Development Setup

1. **Fork the repository**
   ```bash
   git clone https://github.com/topsecretagent007/hyperliquid-trading-bot.git
   cd hyperliquid-trading-bot
   ```

2. **Set up development environment**
   ```bash
   # Install dependencies
   cargo build
   
   # Run tests
   cargo test
   
   # Run clippy for linting
   cargo clippy
   
   # Format code
   cargo fmt
   ```

3. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

## 📋 Contribution Guidelines

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting
- Write comprehensive documentation
- Add tests for new functionality

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add new momentum strategy
fix: resolve order placement bug
docs: update README with new features
test: add unit tests for DCA strategy
```

### Pull Request Process

1. **Create a feature branch** from `main`
2. **Make your changes** with proper tests
3. **Update documentation** if needed
4. **Run all tests** and ensure they pass
5. **Submit a pull request** with a clear description

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass
```

## 🎯 Areas for Contribution

### High Priority
- **New Trading Strategies**: Implement additional trading algorithms
- **Risk Management**: Enhance risk controls and monitoring
- **Performance**: Optimize for speed and memory usage
- **Testing**: Improve test coverage and quality

### Medium Priority
- **Documentation**: Improve guides and examples
- **Monitoring**: Add more metrics and alerts
- **Configuration**: Enhance configuration options
- **Error Handling**: Improve error messages and recovery

### Low Priority
- **UI/UX**: Command-line interface improvements
- **Logging**: Enhanced logging and debugging
- **Examples**: More usage examples and tutorials

## 🧪 Testing Guidelines

### Unit Tests
- Test individual functions and methods
- Mock external dependencies
- Cover edge cases and error conditions
- Aim for >80% code coverage

### Integration Tests
- Test complete workflows
- Use testnet for API testing
- Verify configuration loading
- Test error handling paths

### Example Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dca_strategy_buy_signal() {
        // Test implementation
    }
    
    #[test]
    fn test_risk_management_limits() {
        // Test implementation
    }
}
```

## 📚 Documentation

### Code Documentation
- Use `///` for public API documentation
- Include examples in doc comments
- Document complex algorithms
- Explain business logic

### README Updates
- Update feature lists
- Add new configuration options
- Include new usage examples
- Update installation instructions

## 🐛 Bug Reports

### Before Reporting
1. Check existing issues
2. Test with latest version
3. Verify configuration
4. Check logs for errors

### Bug Report Template
```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g., Windows 10]
- Rust version: [e.g., 1.70.0]
- Bot version: [e.g., 0.1.0]

## Logs
Relevant log output
```

## 💡 Feature Requests

### Before Requesting
1. Check existing issues
2. Consider if it fits the project scope
3. Think about implementation complexity
4. Consider backward compatibility

### Feature Request Template
```markdown
## Feature Description
Clear description of the feature

## Use Case
Why is this feature needed?

## Proposed Solution
How should it be implemented?

## Alternatives Considered
Other approaches considered

## Additional Context
Any other relevant information
```

## 🔒 Security

### Security Issues
- Report security issues privately to [topsecretagent007](https://github.com/topsecretagent007)
- Do not create public issues for security vulnerabilities
- Include detailed reproduction steps
- Allow time for fixes before disclosure

### Security Guidelines
- Never commit API keys or private keys
- Use environment variables for sensitive data
- Validate all inputs
- Follow secure coding practices

## 📞 Getting Help

### Questions and Support
- **GitHub Discussions**: For general questions
- **GitHub Issues**: For bugs and feature requests
- **Telegram**: [@topsecretagent_007](https://t.me/topsecretagent_007)
- **Email**: Contact via GitHub

### Development Help
- Check existing documentation
- Look at similar implementations
- Ask in GitHub discussions
- Join our Telegram group

## 🏆 Recognition

Contributors will be recognized in:
- README contributors section
- Release notes
- Project documentation
- GitHub contributors page

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 🙏 Thank You

Thank you for contributing to the Hyperliquid Trading Bot! Your contributions help make this project better for everyone.

---

**Happy Contributing! 🚀**
