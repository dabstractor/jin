# INI Format Quick Reference

## Basic Syntax

```ini
; This is a comment
# This is also a comment (Unix style)

[SectionName]
key = value
key : value    ; Both = and : work
key =          ; Empty value (valid)

[subsection with dot notation]
key = value
```

## Git Config Subsections

```ini
[section "subsection"]
    key = value

[remote "origin"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
```

## EditorConfig Patterns

```ini
root = true

[*.js]
indent_style = space
indent_size = 2

[src/**/*.ts]
indent_style = space
indent_size = 4
```

## Edge Cases and Solutions

| Problem | Example | Solution |
|---------|---------|----------|
| Equals in value | `key = a=b=c` | Use quotes: `key = "a=b=c"` |
| Multiline value | Spanning lines | Indent continuation or use quotes |
| Duplicate key | `key=1` then `key=2` | Parser dependent; quote values |
| Special chars | `key = value;with;semicolons` | Quote value: `key = "value;with;semicolons"` |
| Reserved words | `enabled = true` | Quote: `enabled = "true"` |
| Backslash in path | Windows path | Use forward slash or escape: `c:\\path\\file` → `"c:\path\file"` |

## Parser Configuration

```javascript
{
  trimValues: true,              // Remove leading/trailing whitespace
  allowDuplicateKeys: false,     // Throw error on duplicates
  allowDuplicateSections: true,  // Merge duplicate sections
  duplicateKeyBehavior: 'override', // 'override' or 'preserve'
  allowSubsections: true,        // Support [section "subsection"]
  commentCharacters: [';', '#'], // Comment delimiters
  caseInsensitive: false,        // Treat as case-sensitive (safer)
  parseBoolean: false,           // Keep as strings (safer)
  strictMode: true,              // Enforce strict validation
}
```

## Merging Strategy

```
Merge Precedence (Git Config Pattern):
  System-wide: /etc/gitconfig
  User global: ~/.gitconfig
  Local repo: .git/config

Merge Function:
  result = base
  for section in override:
    if section exists:
      for key in override[section]:
        result[section][key] = override[section][key]
    else:
      result[section] = override[section]
```

## Best Practices Summary

### DO
- Quote values with special characters
- Use consistent indentation (2-6 spaces)
- Use meaningful section and key names
- Add comments explaining complex settings
- Use dot notation for hierarchy: `[database.connection]`
- Validate parsed results
- Test merge operations in all directions

### DON'T
- Rely on escape sequences
- Use multiline values without parser support
- Mix comment styles
- Use unquoted special characters
- Assume case insensitivity
- Hard-code delimiter handling
- Merge without clear precedence

## Nested Object Mapping

```ini
[database]
host = localhost

[database.connection]
timeout = 30
```

Becomes:

```javascript
{
  "database": {
    "host": "localhost",
    "connection": {
      "timeout": "30"
    }
  }
}
```

## Common Dialects

| Tool | Features | Notes |
|------|----------|-------|
| Standard INI | Sections, keys, values | No subsections, no escape sequences |
| Git Config | Subsections, variables | `[section "subsection"]` syntax |
| EditorConfig | Glob patterns, properties | Path matching, special wildcards |
| PHP parse_ini_file | Basic INI + reserved words | No escape sequences support |
| Python configparser | Full INI + interpolation | Supports multiline, variable expansion |

## Type Handling

```ini
; These might be type-converted by parser:
boolean_true = true      ; May become boolean or "1"
boolean_false = false    ; May become boolean or ""
number = 42              ; May become integer
nullable = null          ; May become ""

; Best practice: Quote to preserve as strings
boolean_true = "true"    ; Stays string "true"
number = "42"            ; Stays string "42"
```

## Validation Checklist

- [ ] Section names contain only [a-zA-Z0-9.-]
- [ ] All keys have a value (even if empty)
- [ ] Special characters in values are quoted
- [ ] No duplicate keys in same section
- [ ] Comments use consistent delimiters
- [ ] Indentation is consistent
- [ ] All sections are properly closed
- [ ] No orphaned key-value pairs (belong to a section)
- [ ] Merged result is valid INI
- [ ] All critical keys are present

## Examples by Use Case

### Application Configuration
```ini
[app]
name = MyApp
debug = false

[database]
host = localhost
port = 3306
```

### Development vs Production
```ini
; base.ini
[server]
port = 8080

; production.ini (override)
[server]
port = 443
ssl_enabled = true
```

### Multi-Section Tool Config
```ini
[core]
    repositoryformatversion = 0
    filemode = true

[user]
    name = Developer
    email = dev@example.com

[remote "origin"]
    url = https://github.com/user/repo.git
```

## Common Parser Error Messages

| Error | Cause | Fix |
|-------|-------|-----|
| Duplicate key in section | Same key twice | Remove duplicate or use array syntax |
| Invalid section header | Missing brackets or spaces | Use `[SectionName]` format |
| Missing delimiter | No = or : between key and value | Add delimiter: `key = value` |
| Unbalanced quotes | Mismatched quote characters | Ensure pairs: `"value"` |
| Invalid section name | Special characters in name | Use alphanumeric, dots, hyphens only |

---

*Quick reference compiled from comprehensive INI format research. See ini_format_patterns.md for detailed documentation.*
