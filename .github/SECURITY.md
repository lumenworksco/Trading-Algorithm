# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

1. **Do NOT** create a public GitHub issue for security vulnerabilities
2. Email security concerns to: [your-email@example.com]
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours of your report
- **Assessment**: Within 7 days, we'll assess the vulnerability and determine its severity
- **Resolution**: We aim to resolve critical vulnerabilities within 30 days
- **Disclosure**: We'll coordinate with you on public disclosure timing

### Scope

Security issues we're interested in:

- Authentication/authorization bypasses
- Code injection vulnerabilities
- Sensitive data exposure
- Cryptographic weaknesses
- Unsafe handling of API credentials

### Out of Scope

- Denial of service attacks
- Social engineering
- Physical attacks
- Issues in dependencies (report to upstream)

### Safe Harbor

We consider security research conducted in good faith to be authorized. We will not pursue legal action against researchers who:

- Make a good faith effort to avoid privacy violations and disruption
- Provide us reasonable time to fix the issue before public disclosure
- Do not exploit the vulnerability beyond what's necessary to demonstrate it

## Security Best Practices for Users

### API Credentials

- Never commit API keys to version control
- Use environment variables for sensitive data
- Rotate API keys regularly
- Use paper trading mode for testing

### Configuration

- Keep configuration files private
- Don't share configuration with API secrets
- Use appropriate file permissions

### Running the Application

- Run with minimal required permissions
- Monitor for unusual activity
- Keep the application updated

## Acknowledgments

We appreciate the security research community and will acknowledge researchers who help improve our security (with their permission).
