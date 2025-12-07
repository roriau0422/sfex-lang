# Security Policy

## Supported Versions

We actively support the following versions of SFX with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| 0.2.x   | :x:                |
| 0.1.x   | :x:                |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in SFX, please follow these steps:

### 1. **DO NOT** Create a Public Issue

Please do not open a public GitHub issue if the bug is a security vulnerability.

### 2. Report Privately

Send an email to: **roriau@gmail.com**

Include the following information:
- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if any)

### 3. Response Timeline

- **Initial Response**: Within 48 hours of report
- **Status Update**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: Within 7 days
  - High: Within 14 days
  - Medium: Within 30 days
  - Low: Next release cycle

### 4. Disclosure Policy

- We follow **responsible disclosure**
- We will coordinate with you on the disclosure timeline
- Typically 90 days after the fix is released
- Credit will be given to reporters (unless you prefer anonymity)

## Security Considerations

### What to Report

**DO report:**
- Command injection vulnerabilities
- Path traversal issues
- Memory safety issues (despite Rust's safety)
- Arbitrary code execution
- Privilege escalation
- Information disclosure
- Denial of service (DoS) vulnerabilities

**Examples in SFX context:**
- Unsafe file operations in `File` module
- Command injection in `System.Execute`
- Resource exhaustion in JIT compiler
- Memory leaks in reactive observers
- Unsafe FFI in JIT compiled code

### What NOT to Report

**Please don't report:**
- Issues with third-party dependencies (report to them directly)
- Social engineering attacks
- Physical security issues
- Theoretical vulnerabilities without proof of concept

## Security Best Practices

When using SFX, follow these guidelines:

### 1. Input Validation
```sfex
# Always validate user input
UserInput is File.Read("user_data.txt")

# Sanitize before use
If UserInput contains "../":
    Print "Invalid path detected!"
    Return
```

### 2. File Operations
```sfex
# Use absolute paths when possible
SafePath is "/home/user/data/file.txt"

# Avoid user-controlled paths
# BAD: File.Read(UserInput)
# GOOD: File.Read(SafePath)
```

### 3. System Commands
```sfex
# Never pass unsanitized user input to System.Execute
# BAD: System.Execute("ls " + UserInput)

# GOOD: Use allowlist
AllowedCommands is ["status", "version", "help"]
If AllowedCommands contains UserCommand:
    System.Execute(UserCommand)
```

### 4. Network Operations
```sfex
# Validate URLs before fetching
Url is "https://trusted-domain.com/api"

# Don't trust external data
Response is HTTP.Get(Url)
# Parse and validate response before use
```

### 5. Concurrent Code
```sfex
# Be careful with shared state in concurrent code
# Use proper synchronization
Task1 is Do in background:
    # Access shared resources carefully
```

## Known Security Limitations

### 1. No Sandboxing
SFX code runs with the same privileges as the interpreter. There is no built-in sandboxing.

### 2. File System Access
The `File` module has unrestricted file system access. Use caution when running untrusted code.

### 3. System Commands
`System.Execute` can run arbitrary shell commands. Never pass unsanitized user input.

### 4. JIT Compilation
The JIT compiler generates native code. While Cranelift is used, untrusted code should be reviewed.

### 5. Networking
HTTP, WebSocket, TCP, and UDP modules can access any network resource. Use firewall rules if needed.

## Security Updates

Security fixes are released as:
- **Patch releases** (0.3.x) for non-breaking fixes
- **Minor releases** (0.x.0) for breaking security improvements
- **Backports** to supported versions when feasible

Stay updated:
- Watch the repository for security announcements
- Subscribe to GitHub Security Advisories
- Check the [Releases page](https://github.com/roriau0422/sfex-lang/releases)

## Security Hall of Fame

We recognize security researchers who help improve SFX:

<!-- Will be updated as security reports are received and fixed -->

*No reports yet - be the first!*

## Additional Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Apache 2.0 License](LICENSE)

## Questions?

For security-related questions that are not vulnerabilities, you can:
- Open a [GitHub Discussion](https://github.com/roriau0422/sfex-lang/discussions)
- Create an issue with the `security` label
- Contact: roriau@gmail.com

---

**Thank you for helping keep SFX and its users safe!**
