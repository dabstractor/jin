# INI File Format Research - P2M2 Module

Comprehensive research on INI file format parsing, edge cases, and merging strategies for the P2M2 MergeValue system.

## Documents Overview

This research directory contains four comprehensive documents covering all aspects of INI file format handling:

### 1. **ini_format_patterns.md** (1066 lines)
The primary comprehensive research document covering:
- INI format specification (sections, keys, values, comments)
- Common patterns in developer tools (Git config, EditorConfig)
- Nested object mapping strategies
- 9 categories of edge cases with handling strategies
- Merging best practices and strategies
- Parser implementation considerations
- Practical examples by use case
- Best practices summary
- Complete references and sources

**Use this for:** Deep understanding of INI format, edge case handling, implementation details

### 2. **ini_quick_reference.md** (219 lines)
Quick reference guide with:
- Basic syntax examples
- Git config subsection patterns
- EditorConfig patterns
- Edge cases with solutions (table format)
- Parser configuration options
- Merging strategy diagrams
- Best practices summary (DO's and DON'Ts)
- Nested object mapping example
- Common INI dialects comparison
- Type handling guidelines
- Validation checklist

**Use this for:** Quick lookup during implementation, syntax reference, validation checks

### 3. **ini_code_examples.md** (671 lines)
Practical code implementations including:
- Complete JavaScript INIParser class (full implementation)
- INIMerger class with multiple merge strategies
- Real-world examples:
  - Git config complete example
  - EditorConfig example with glob patterns
  - Application config (base + dev + prod overrides)
- Testing examples with test cases
- Edge case handling demonstrations
- Production-ready code patterns

**Use this for:** Implementation reference, code templates, testing patterns

### 4. **README.md** (this file)
Navigation and overview document.

---

## Key Research Findings

### 1. Format Specification

**No RFC Standard Exists**
- INI is an informal format with many dialects
- Different tools (Git, EditorConfig, PHP, Python) have variations
- Parser implementation often defines the specification for that tool

**Basic Structure**
```ini
[SectionName]           ; Section header
key = value             ; Key-value pair
; Comment               ; Comment (semicolon)
# Comment               ; Comment (hash, Unix style)
```

**Delimiters**
- Equals (`=`) and colon (`:`) both work
- First occurrence is the delimiter
- Values can contain subsequent delimiters if quoted

### 2. Developer Tool Patterns

**Git Config**
- Supports subsections: `[section "subsection"]`
- Three-level hierarchy: system → user → local
- Escaping required for special chars in subsection names

**EditorConfig**
- Sections are glob patterns (file matchers)
- Supports wildcards and character classes
- Upward directory search with precedence

**Standard INI**
- Simple flat structure with sections
- Dot notation used for logical grouping: `[database.connection]`

### 3. Nested Object Mapping

**Two Approaches:**

1. **Subsection Syntax (Git Config):**
```ini
[remote "origin"]
    url = value
```
Maps to: `config.remote.origin.url`

2. **Dot Notation (Flattening):**
```ini
[database.connection]
    host = value
```
Maps to: `config.database.connection.host`

### 4. Edge Cases (9 Categories)

| # | Edge Case | Solution |
|---|-----------|----------|
| 1 | Equals in values | Quote values: `"value=with=equals"` |
| 2 | Multiline values | Indentation continuation or quotes |
| 3 | Duplicate keys | Parser configured behavior (last-wins, first-wins, error) |
| 4 | Duplicate sections | Merge properties from all occurrences |
| 5 | Reserved words | Quote to prevent type conversion |
| 6 | Missing equals | Entries ignored (require `key = ` syntax) |
| 7 | Whitespace values | Trimmed to empty string by default |
| 8 | Escape sequences | NOT universally supported (avoid relying on) |
| 9 | Case sensitivity | Platform-dependent (treat as case-sensitive) |

### 5. Merging Strategies

**Precedence Model (Git Config Pattern):**
```
System-wide config ← User global ← Repository local
(Lowest priority)                    (Highest priority)
```

**Merge Types:**
1. **Section-level:** Entire sections replaced
2. **Key-level:** Individual keys merged, source wins
3. **Deep merge:** Recursive for nested objects

**Validation Checklist:**
- Preserve unmodified keys from base
- Add new keys from override
- Override matching keys with source values
- Merge duplicate sections
- Validate result structure
- Preserve comments (when possible)

### 6. Parser Configuration Recommendations

```javascript
{
  trimValues: true,              // Remove whitespace
  allowDuplicateKeys: false,     // Detect duplicates
  allowDuplicateSections: true,  // Merge duplicate sections
  duplicateKeyBehavior: 'override', // Last-wins strategy
  allowSubsections: true,        // Support [section "subsection"]
  commentCharacters: [';', '#'], // Both comment styles
  caseInsensitive: false,        // Treat as case-sensitive
  parseBoolean: false,           // Keep as strings (safer)
  strictMode: true,              // Enforce strict validation
}
```

---

## Implementation Checklist for P2M2

### Parser Requirements
- [ ] Parse section headers: `[SectionName]` and `[section "subsection"]`
- [ ] Parse key-value pairs with `=` or `:` delimiters
- [ ] Support quoted values (single and double quotes)
- [ ] Handle comments (`;` and `#`)
- [ ] Trim whitespace around delimiters
- [ ] Preserve comments and formatting for display
- [ ] Validate section/key names

### Edge Case Handling
- [ ] Quote values with special characters automatically
- [ ] Detect and handle duplicate sections (merge)
- [ ] Detect and handle duplicate keys (configurable)
- [ ] Handle empty values
- [ ] Support multiline values (optional)
- [ ] Validate against reserved words
- [ ] Handle case sensitivity per dialect

### Merging Features
- [ ] Section-level merge (atomic)
- [ ] Key-level merge (granular)
- [ ] Three-level precedence support (system/user/local)
- [ ] Validation of merged results
- [ ] Preserve comments during merge
- [ ] Rollback capability
- [ ] Logging/audit trail

### Testing Coverage
- [ ] Unit tests for all edge cases
- [ ] Integration tests for merge operations
- [ ] Format preservation tests
- [ ] Compatibility tests with real INI files
- [ ] Performance tests for large files

---

## Best Practices Summary

### DO:
- Quote values containing special characters
- Use consistent indentation (2-6 spaces)
- Use meaningful, descriptive key names
- Validate parsed and merged results
- Test merge in all directions
- Preserve comments during merge
- Define clear merge precedence
- Handle errors gracefully with context

### DON'T:
- Rely on escape sequences (not universal)
- Mix comment styles in same file
- Use unquoted special characters
- Assume case insensitivity
- Merge without clear precedence
- Lose error context in exceptions
- Modify values during parsing
- Silently discard configuration

---

## Sources and References

**Primary Standards & Documentation:**
- [INI file - Wikipedia](https://en.wikipedia.org/wiki/INI_file)
- [Git Configuration Documentation](https://git-scm.com/docs/git-config)
- [EditorConfig Specification](https://spec.editorconfig.org/index.html)
- [FileFormat.com - INI Format](https://docs.fileformat.com/system/ini/)

**Programming Language References:**
- [Python configparser - Python Docs](https://docs.python.org/3/library/configparser.html)
- [PHP parse_ini_file - PHP Manual](https://www.php.net/manual/en/function.parse-ini-file.php)

**Parser Projects:**
- [ini-parser Wiki - Configuring Behavior](https://github.com/rickyah/ini-parser/wiki/Configuring-parser-behavior)
- [ini-merge - Rust Implementation](https://github.com/VorpalBlade/ini-merge)
- [inifile - Ruby Package](https://www.rubydoc.info/gems/inifile)

**Discussions & Issues:**
- [ini-parser Issues #63, #69, #179, #201](https://github.com/rickyah/ini-parser/issues)

---

## Quick Navigation

| Need | Document | Section |
|------|----------|---------|
| Understand INI format | ini_format_patterns.md | Section 1 |
| Git config details | ini_format_patterns.md | Section 2.1 |
| EditorConfig patterns | ini_format_patterns.md | Section 2.2 |
| Nested object mapping | ini_format_patterns.md | Section 3 |
| Handle equals in value | ini_quick_reference.md | Edge Cases table |
| Handle multiline values | ini_format_patterns.md | Section 4.2 |
| Parser configuration | ini_quick_reference.md | Parser Configuration |
| Code implementation | ini_code_examples.md | All |
| Test cases | ini_code_examples.md | Testing Examples |
| Merging strategies | ini_format_patterns.md | Section 5 |
| Validation checklist | ini_quick_reference.md | Validation Checklist |

---

## File Organization

```
plan/P2M2/research/
├── README.md                      (this file - navigation & overview)
├── ini_format_patterns.md         (comprehensive research - 1066 lines)
├── ini_quick_reference.md         (quick lookup guide - 219 lines)
└── ini_code_examples.md           (implementation examples - 671 lines)
```

**Total: 1,956 lines of detailed research documentation**

---

## Usage Recommendations

### For Initial Implementation
1. Start with **ini_quick_reference.md** for syntax overview
2. Review **ini_code_examples.md** for parser structure
3. Reference **ini_format_patterns.md** for specific edge cases

### For Edge Case Handling
1. Check **ini_quick_reference.md** for quick solution
2. Read detailed explanation in **ini_format_patterns.md** Section 4
3. See code example in **ini_code_examples.md**

### For Merging Logic
1. Review merging strategies in **ini_quick_reference.md**
2. Study implementation in **ini_code_examples.md** (INIMerger class)
3. Reference best practices in **ini_format_patterns.md** Section 5

### For Testing
1. Use test cases from **ini_code_examples.md**
2. Add edge case tests for your dialect
3. Validate against real INI files (Git config, EditorConfig)

---

*Research compiled from authoritative sources including Wikipedia, official documentation (Git, EditorConfig, PHP, Python), and major parser projects. Last updated: 2025-12-27*
