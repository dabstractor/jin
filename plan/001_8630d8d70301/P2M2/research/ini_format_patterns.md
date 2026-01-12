# INI File Format Parsing and Merging - Comprehensive Research

## Executive Summary

INI (Initialization) files are a simple, informal text-based configuration format with no formal RFC standard. Multiple dialects exist across different tools and frameworks. This document provides a comprehensive overview of INI format specifications, common patterns in developer tools, edge cases, and merging best practices for proper INI file handling.

---

## 1. INI File Format Specification

### 1.1 Basic Structure

INI files consist of three fundamental components:
- **Sections**: Groupings of related key-value pairs
- **Keys (Properties)**: Unique identifiers for configuration values
- **Values**: The actual configuration data
- **Comments**: Lines or inline notes explaining the configuration

### 1.2 Sections

**Syntax:**
```ini
[SectionName]
```

**Rules:**
- Section names appear on their own line, enclosed in square brackets `[` and `]`
- Section names are case-insensitive in Windows, case-sensitive in Unix
- Only alphanumeric characters, hyphens (`-`), and dots (`.`) are allowed in standard INI section names
- All key-value pairs following a section header belong to that section until a new section is declared
- Sections cannot be arbitrarily nested syntactically

**Example:**
```ini
[user]
[database]
[server]
```

### 1.3 Keys and Values

**Syntax:**
```ini
key = value
key : value
```

**Rules:**
- Keys and values are separated by an equals sign (`=`) or colon (`:`)
- Both delimiters are commonly supported across INI parsers
- Whitespace around the delimiter is typically trimmed
- Keys must have a delimiter (entries without `=` or `:` are typically ignored)
- Keys are case-insensitive in Windows, case-sensitive in Unix

**Examples:**
```ini
name = John Doe
email = john@example.com
port: 3306
debug = true
```

### 1.4 Comments

**Syntax:**
```ini
; This is a comment
# This is also a comment
key = value ; inline comment (some parsers support this)
```

**Rules:**
- Semicolon (`;`) is the primary comment character
- Hash/octothorpe (`#`) is often supported as well (Unix shell style)
- Comments extend to the end of the line
- Some parsers support inline comments after values
- Blank lines are typically allowed and ignored

### 1.5 Character Encoding and Line Endings

- **Encoding**: ISO 8859-1 (Latin-1) or UTF-8 with BOM in modern implementations
- **Line Endings**:
  - Traditionally CRLF (Windows style)
  - LF (Unix style) is also widely supported
  - Some modern implementations require UTF-8 with CRLF or LF

### 1.6 Whitespace Handling

- Leading and trailing whitespace around keys, values, and delimiters is typically trimmed
- Indentation can be used for readability (typically 2-6 spaces, consistently)
- Whitespace within values is preserved (except leading/trailing)

---

## 2. Common INI File Patterns in Developer Tools

### 2.1 Git Config INI Format

**File Locations (Precedence Order):**
1. System-wide: `/etc/gitconfig`
2. User/Global: `~/.gitconfig` or `~/.config/git/config`
3. Repository-local: `.git/config`

**Key Features:**
- Supports subsections with quoted names
- Variables can belong directly to a section or to a subsection
- Case-insensitive section names, case-sensitive subsection names

**Subsection Syntax:**
```ini
[section "subsection"]
    key = value
```

**Complete Example:**
```ini
[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true

[user]
    name = Your Name
    email = your.email@example.com

[remote "origin"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*

[remote "upstream"]
    url = https://github.com/original/repo.git
    fetch = +refs/heads/*:refs/remotes/upstream/*

[branch "main"]
    remote = origin
    merge = refs/heads/main

[alias]
    st = status
    co = checkout
    br = branch
```

**Key Git Config Characteristics:**
- Subsection names are case-sensitive
- Can contain any characters except newline and null byte
- Double quotes and backslashes in subsection names must be escaped: `\"` and `\\`
- Three-level hierarchy: section > subsection > key-value
- Values with special characters must be quoted

### 2.2 EditorConfig INI Format

**File Name:** `.editorconfig`

**Key Features:**
- Section names are filename glob patterns (not literal section names)
- More complex than standard INI due to pattern matching
- Forward slashes (`/`) as path separators only
- Backslashes are NOT allowed as path separators

**Glob Pattern Syntax:**
- `*` - Matches any string except path separators
- `**` - Matches any string including path separators
- `?` - Matches any single character
- `[seq]` - Character class matching
- `[!seq]` - Negated character class
- `{s1,s2,s3}` - Alternation
- `{num1..num2}` - Numeric ranges
- Backslash escapes special characters

**File Discovery:**
- Searches from the file's directory upward through parent directories
- Files are read top to bottom
- Most recent (closest to root) rules take precedence

**Complete Example:**
```ini
# EditorConfig helps maintain consistent coding styles
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.{js,ts,jsx,tsx}]
indent_style = space
indent_size = 2

[*.py]
indent_style = space
indent_size = 4

[*.{json,yaml,yml}]
indent_style = space
indent_size = 2

[Makefile]
indent_style = tab

[*.md]
trim_trailing_whitespace = false
```

**Property Constraints:**
- Property names are case-insensitive (lowercased during parsing)
- Maximum property name length: 50 characters
- Maximum property value length: 255 characters
- Maximum section name length: 4096 characters

---

## 3. INI Sections to Nested Objects Mapping

### 3.1 Conceptual Mapping

**INI Structure:**
```ini
[section]
key1 = value1
key2 = value2

[section.subsection]
key3 = value3
```

**Maps To JavaScript/JSON Object:**
```javascript
{
  "section": {
    "key1": "value1",
    "key2": "value2"
  },
  "section.subsection": {  // Single-level nesting with dot notation
    "key3": "value3"
  }
}
```

### 3.2 Two-Level Mapping (Subsections)

**Git Config INI:**
```ini
[remote "origin"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
```

**Maps To Nested Object:**
```javascript
{
  "remote": {
    "origin": {
      "url": "https://github.com/user/repo.git",
      "fetch": "+refs/heads/*:refs/remotes/origin/*"
    }
  }
}
```

### 3.3 Dot-Notation Flattening Pattern

For tools that don't support subsections, nesting is achieved through dot notation in section names:

**INI with Dot Notation:**
```ini
[database.connection]
host = localhost
port = 3306

[database.credentials]
username = admin
password = secret

[server.ssl]
enabled = true
certificate = /path/to/cert.pem
```

**Maps To Nested Object:**
```javascript
{
  "database": {
    "connection": {
      "host": "localhost",
      "port": "3306"
    },
    "credentials": {
      "username": "admin",
      "password": "secret"
    }
  },
  "server": {
    "ssl": {
      "enabled": "true",
      "certificate": "/path/to/cert.pem"
    }
  }
}
```

### 3.4 Parsing Strategy for Nested Objects

```pseudocode
Parse INI File:
  1. For each section [name]:
     - Extract parent section and child subsection (if dot notation)
     - Create nested object structure in result

  2. For each key = value:
     - Add key-value pair to current section's object
     - Preserve types if type inference enabled (true, false, numbers)

Nested Object Structure:
  result[section] = {}
  result[section][key] = value

  For subsections:
  result[section][subsection] = {}
  result[section][subsection][key] = value
```

---

## 4. Edge Cases and Handling Strategies

### 4.1 Equals Signs in Values

**Problem:** Values containing equals signs are ambiguous

**Solutions:**

1. **Quoting Values:**
```ini
; Solution: Quote the entire value
connection_string = "Server=localhost;Port=5432;User=admin"
math_expression = "x=5 AND y=10"
```

2. **Base64 Encoding (for complex values):**
```ini
; When value contains special characters including =
encoded_data = "aW52YWxpZF92YWx1ZXM9dGhlc2U="
```

3. **First Equals as Delimiter (Most Common):**
```ini
; Parser takes only the FIRST = as delimiter
key = value=with=equals = should work
; Result: key = "value=with=equals = should work"
```

**Best Practice:** Quote values containing special characters
```ini
connection_string = "Server=localhost;Port=3306"
calculation = "2+2=4"
```

### 4.2 Multiline Values

**Problem:** Values spanning multiple lines

**Solutions:**

1. **Indentation Continuation (if supported):**
```ini
[software_licenses]
license_text = This is a long license that spans
    multiple lines by indenting
    continuation lines
```

2. **Value Concatenation:**
```ini
[paths]
config_path = /etc/app \
    /config \
    /settings.ini
```

3. **Escape Sequences:**
```ini
[text]
; Note: Not all parsers support escape sequences
message = Line 1\nLine 2\nLine 3
```

**Important Note:** Multiline value support varies significantly by parser. Python's ConfigParser supports indented continuation. PHP's parse_ini_file does NOT support standard escape sequences like \n or \t.

### 4.3 Duplicate Keys

**Problem:** Same key appears multiple times in a section

**Different Parser Behaviors:**

1. **Last-Value-Wins (Most Common):**
```ini
[settings]
timeout = 30
timeout = 60
; Result: timeout = 60
```

2. **First-Value-Wins:**
```ini
[settings]
timeout = 30
timeout = 60
; Result: timeout = 30
```

3. **Error/Exception:**
```ini
[settings]
timeout = 30
timeout = 60
; Result: Parser raises DuplicateOptionError
```

4. **Array/List Accumulation (Some Parsers):**
```ini
[includes]
file = config1.ini
file = config2.ini
file = config3.ini
; Result: file = ["config1.ini", "config2.ini", "config3.ini"]
```

**Best Practice:** Avoid duplicate keys. Use parser configuration to detect duplicates during validation.

### 4.4 Duplicate Sections

**Problem:** Same section header appears multiple times

**Handling:**
```ini
[database]
host = localhost
port = 3306

[other_settings]
timeout = 30

[database]
username = admin
password = secret
; Result: Depends on parser
; Some merge properties: database = {host, port, username, password}
; Others: Second section overwrites first
```

**Most Parsers:** Merge properties from duplicate sections into a single section object.

### 4.5 Reserved Words / Type Conversion

**Reserved Words (Some Parsers):**
```ini
; These values may be interpreted as types, not strings:
; null, yes, no, true, false, on, off, none

[booleans]
enabled = true        ; Converted to boolean true
disabled = false      ; Converted to boolean false
flag = yes           ; Converted to boolean or "1"
nullable = null      ; Converted to empty string ""
```

**PHP Behavior (with INI_SCANNER_TYPED):**
- `null`, `off`, `no`, `false` → empty string `""`
- `on`, `yes`, `true` → string `"1"`
- Numeric strings → integers
- Others → remain strings

**Best Practice:** Quote values to prevent unintended type conversion:
```ini
enabled = "true"      ; Remains string "true"
count = "42"          ; Remains string "42"
```

### 4.6 Missing Equals Sign

**Problem:** Entries without delimiters

**Behavior:**
```ini
[section]
key_without_value
key = value
property_alone
; Result: Entries without = are ignored
; Only "key = value" is parsed
```

**Valid Entry Without Value:**
```ini
[section]
key =                 ; Empty value (valid)
key =  ; Result: key = ""
```

### 4.7 Entries with Only Whitespace Values

**Problem:** Ambiguity with whitespace preservation

**Behavior:**
```ini
[section]
key1 =
key2 =
key3 = value
; key1 and key2 are empty strings ""
; Whitespace-only values are typically trimmed to empty
```

### 4.8 Escape Sequences

**Supported by Some Parsers:**
```ini
[text]
; ANSI C escape sequences (if supported):
newline = "Line 1\nLine 2"
tab = "Column1\tColumn2"
backslash = "Path\\to\\file"
quote = "She said \"hello\""
```

**Important Caveats:**
- PHP's parse_ini_file does NOT support `\n`, `\t`, etc.
- Most parsers only support escaping in quoted strings
- Backslash itself must be escaped: `\\`
- Use stripcslashes() in PHP if needed

**Best Practice:** Don't rely on escape sequences. Use different approaches:
- For multiline: Use parser that supports continuation
- For paths: Use forward slashes (work cross-platform)
- For special chars: Use quoting

### 4.9 Case Sensitivity

**Section Names:**
- Windows INI: Case-insensitive (normalized to lowercase)
- Unix-style: Case-sensitive
- Git config: Section names case-insensitive, subsection names case-sensitive

**Key Names:**
- Windows INI: Case-insensitive
- Unix-style: Case-sensitive (more common in modern usage)

**Best Practice:** Treat as case-sensitive to ensure cross-platform compatibility.

---

## 5. INI File Merging Best Practices

### 5.1 Merging Strategies Overview

Two primary approaches for INI file merging:

#### 5.1.1 Section-Level Merge
Entire sections are merged or replaced atomically.

```ini
; base.ini
[database]
host = localhost
port = 3306

; override.ini
[database]
port = 5432
password = secret

; Result after merge:
[database]
host = localhost
port = 5432          ; Overwritten
password = secret    ; Added
```

#### 5.1.2 Key-Level Merge
Individual keys are merged, with source values preferred.

```
Same as section-level in this example, but source (override.ini)
values override target (base.ini) values for matching keys.
```

### 5.2 Merge Direction and Precedence

**Precedence Hierarchy (Git Config Pattern):**
```
1. Repository-local (.git/config)
2. User/Global (~/.gitconfig)
3. System-wide (/etc/gitconfig)
```

**Application Pattern:**
```
merged_config = merge(
  system_config,       // Base level
  merge(
    user_config,       // Middle level
    repo_config        // Highest priority
  )
)
```

**Asymmetric Merging Rules:**
- Source INI values are preferred unless specific rules exist
- Existing keys in target are overwritten by source
- New keys in source are added to target
- Keys only in target are preserved (no deletion)

### 5.3 Handling Conflicts

**Duplicate Sections Across Files:**
```ini
; config1.ini
[server]
port = 8080
host = 0.0.0.0

; config2.ini
[server]
port = 9000
timeout = 30

; Merge Result:
[server]
port = 9000          ; config2 wins
host = 0.0.0.0       ; from config1
timeout = 30         ; from config2
```

**Duplicate Keys Within Section:**

Strategy 1: Last-write-wins
```
When merging files, later files override earlier ones
```

Strategy 2: Deep merge
```
For complex sections, recursively merge sub-objects
```

Strategy 3: Explicit conflict resolution
```
Define rules for which file takes precedence for specific keys
```

### 5.4 Preservation of Format and Comments

**Challenge:** Maintaining original formatting and comments during merge

**Approaches:**

1. **Parse → Modify → Rewrite:**
   - Keeps document structure
   - Preserves comments and formatting
   - More complex implementation

2. **Simple Key-Value Merge:**
   - Creates new document from merged values
   - Loses original comments and formatting
   - Simpler implementation

**Recommendation:** Preserve comments and formatting when possible:
```javascript
// Pseudocode
const result = parseAndMerge(baseFile, overrideFile);
result.preserveComments = true;
result.preserveFormatting = true;
return result.toString();
```

### 5.5 Implementing a Merge Function

**Pseudocode for Section-Level Merge:**

```
function mergeINI(baseINI, overrideINI):
  result = copy(baseINI)

  for each section in overrideINI:
    if section exists in result:
      // Merge keys in section
      for each key in overrideINI[section]:
        result[section][key] = overrideINI[section][key]
    else:
      // New section, add entirely
      result[section] = copy(overrideINI[section])

  return result
```

**Pseudocode for Deep Merge (with nested objects):**

```
function deepMerge(base, override):
  result = copy(base)

  for each key in override:
    if key in result AND isObject(override[key]):
      // Recursively merge nested objects
      result[key] = deepMerge(result[key], override[key])
    else:
      // Overwrite or add new key
      result[key] = override[key]

  return result
```

### 5.6 Special Considerations for Different Tools

**Git Config Merging:**
```
- System level (/etc/gitconfig) merged with
- Global level (~/.gitconfig) merged with
- Local level (.git/config)
- Each level completely overrides previous for matching keys
```

**EditorConfig Merging:**
```
- Search upward through parent directories
- All matching .editorconfig files are processed
- Closer files override farther files
- Glob patterns determine which rules apply to which files
```

**Application Config Merging:**
```
- Base configuration file (defaults)
- Environment-specific overrides (dev, staging, prod)
- User-local overrides (.gitignored)
- Command-line arguments (highest priority)
```

### 5.7 Merge Validation

**Checklist for Merge Implementation:**

- [ ] Preserve all keys from base that aren't in override
- [ ] Add all new keys from override
- [ ] Override matching keys with override values
- [ ] Handle duplicate sections by merging properties
- [ ] Validate merged result has valid INI structure
- [ ] Test with empty sections
- [ ] Test with duplicate sections in same file
- [ ] Test with special characters in values
- [ ] Test with multiline values (if supported)
- [ ] Ensure proper type handling for values
- [ ] Validate section and key names
- [ ] Test merge direction precedence

---

## 6. Parser Implementation Considerations

### 6.1 Key Parsing Steps

1. **Tokenization:**
   - Split file by lines
   - Identify comments and remove them
   - Identify blank lines

2. **Section Detection:**
   - Match `[SectionName]` pattern
   - Extract section name
   - Handle subsection syntax for Git config

3. **Key-Value Extraction:**
   - Match `key = value` or `key : value` pattern
   - Extract key (left of delimiter)
   - Extract value (right of delimiter)
   - Trim whitespace as appropriate
   - Handle quoted values

4. **Type Inference (Optional):**
   - Detect boolean values (true/false, yes/no, on/off)
   - Detect numeric values (integers, floats)
   - Keep everything as strings if strict mode

5. **Output Formatting:**
   - Build nested object structure
   - Assign values to correct sections
   - Return parsed configuration

### 6.2 Configuration Options for Parsers

**Recommended Parser Options:**
```javascript
{
  // Whitespace handling
  trimValues: true,           // Trim leading/trailing from values
  allowWhitespaceInKeys: false, // Keys shouldn't have spaces

  // Comment handling
  commentCharacters: [';', '#'], // Allow both comment styles

  // Case handling
  caseInsensitive: false,     // Treat as case-sensitive (safer)

  // Type inference
  parseBoolean: false,        // Keep everything as strings (safer)
  parseNumbers: false,        // Keep everything as strings (safer)

  // Duplicate handling
  allowDuplicateKeys: false,  // Throw error on duplicates
  allowDuplicateSections: true, // Allow and merge duplicate sections
  duplicateKeyBehavior: 'override', // last-write-wins

  // Multiline handling
  allowMultilineValues: true, // Support continued lines
  multilineBracket: false,    // Don't use brackets for multiline

  // Subsection support
  allowSubsections: true,     // Support [section "subsection"]

  // Validation
  strictMode: true,           // Enforce strict INI rules
}
```

### 6.3 Error Handling

**Common Parse Errors:**
- Invalid section header format
- Duplicate keys in same section
- Duplicate section names
- Missing delimiter in key-value pair
- Invalid characters in section names
- Unbalanced quotes
- Invalid escape sequences

---

## 7. Practical Examples by Use Case

### 7.1 Application Configuration

**Pattern: Environment-based configuration**

```ini
; config/default.ini
[app]
name = MyApp
version = 1.0.0
debug = false

[database]
host = localhost
port = 3306
database = myapp_default

[logging]
level = INFO
format = json

; config/development.ini (overrides default.ini)
[app]
debug = true

[database]
database = myapp_dev

[logging]
level = DEBUG

; config/production.ini (overrides default.ini)
[app]
debug = false

[database]
host = prod.db.example.com
port = 5432
database = myapp_prod

[logging]
level = ERROR
```

**Merge Flow:**
```
default.ini ← development.ini (when ENV=development)
default.ini ← production.ini (when ENV=production)
```

### 7.2 Tool Configuration

**Pattern: Tool with base + local overrides**

```ini
; .editorconfig (project-wide)
root = true

[*]
charset = utf-8
end_of_line = lf
trim_trailing_whitespace = true

[*.js]
indent_style = space
indent_size = 2

; .editorconfig.local (local developer preferences, .gitignored)
[*.js]
indent_size = 4  ; Developer prefers 4 spaces
```

### 7.3 Multi-File Configuration System

**Pattern: System + User + Local**

```
/etc/myapp/config.ini         (system-wide defaults)
~/.myapp/config.ini           (user preferences)
./myapp.ini                   (project/directory-specific)
./myapp.local.ini             (local overrides, .gitignored)
```

**Precedence:**
```
system ← user ← project ← local (highest priority)
```

---

## 8. Summary of Best Practices

### 8.1 Format Best Practices

✓ **DO:**
- Use uppercase section names for clarity: `[DATABASE]`
- Use lowercase key names: `hostname`
- Quote values containing special characters: `"value=with=equals"`
- Use consistent indentation (2-6 spaces)
- Add comments explaining complex settings
- Use dot notation for logical grouping: `[database.connection]`
- Use meaningful, descriptive key names
- Group related settings in sections

✗ **DON'T:**
- Rely on escape sequences (not universally supported)
- Use multiline values without parser support confirmation
- Mix comment styles (pick ; or # and stick with it)
- Use unquoted values with special characters
- Create deeply nested sections (2 levels max)
- Use reserved words as values without quoting
- Mix indentation styles

### 8.2 Parsing Best Practices

✓ **DO:**
- Validate section and key names
- Trim whitespace from values
- Support both `=` and `:` delimiters
- Detect and report duplicate keys/sections
- Handle quoted values properly
- Test with Unicode/non-ASCII characters
- Provide configuration options for parser behavior
- Use strict mode for validation

✗ **DON'T:**
- Assume escape sequences work
- Ignore whitespace handling
- Skip validation of input
- Assume case insensitivity
- Hard-code special character handling
- Modify values during parsing (preserve as-is)
- Lose error context in exception messages

### 8.3 Merging Best Practices

✓ **DO:**
- Clearly define merge precedence
- Document which file overrides which
- Preserve comments during merge (when possible)
- Validate merged result
- Test merge in all directions
- Handle empty sections/keys
- Log merge operations for debugging
- Provide rollback capability

✗ **DON'T:**
- Silently lose configuration values
- Merge without clear precedence rules
- Assume commutative merging (order matters)
- Merge without validation
- Discard important comments/formatting
- Merge incompatible INI dialects without translation
- Perform in-place merges without backups

---

## 9. References and Source Materials

### Primary Sources

1. [INI file - Wikipedia](https://en.wikipedia.org/wiki/INI_file)
2. [INI - Initiation File Format - FileFormat.com](https://docs.fileformat.com/system/ini/)
3. [Git Configuration Documentation](https://git-scm.com/docs/git-config)
4. [EditorConfig Format Specification](https://spec.editorconfig.org/index.html)
5. [EditorConfig Documentation](https://docs.editorconfig.org/en/master/editorconfig-format.html)
6. [PHP parse_ini_file Manual](https://www.php.net/manual/en/function.parse-ini-file.php)
7. [Python configparser Documentation](https://docs.python.org/3/library/configparser.html)
8. [INI Parser GitHub Wiki - Configuring Behavior](https://github.com/rickyah/ini-parser/wiki/Configuring-parser-behavior)

### Key GitHub Issues and Discussions

- [rickyah/ini-parser Issue #69 - Duplicate Keys](https://github.com/rickyah/ini-parser/issues/69)
- [rickyah/ini-parser Issue #63 - Duplicate Key Support](https://github.com/rickyah/ini-parser/issues/63)
- [rickyah/ini-parser Issue #201 - Multiline Values](https://github.com/rickyah/ini-parser/issues/201)
- [rickyah/ini-parser Issue #179 - Escaping Equals Sign](https://github.com/rickyah/ini-parser/issues/179)

### Related Projects

- [ini-merge: INI Merging Crate (Rust)](https://github.com/VorpalBlade/ini-merge)
- [git-config-ini: npm Package](https://www.npmjs.com/package/git-config-ini)
- [inifile: Ruby Package](https://www.rubydoc.info/gems/inifile/3.0.0)

---

## 10. Implementation Recommendations for P2M2

Based on this research, for the P2M2 MergeValue system handling INI files:

### Key Implementation Points

1. **Parser Requirements:**
   - Support both `=` and `:` delimiters
   - Handle quoted values (both single and double quotes)
   - Support Git config subsection syntax: `[section "subsection"]`
   - Trim whitespace around delimiters
   - Preserve comments and formatting for merge operations

2. **Merging Requirements:**
   - Implement section-level merging with key-level overrides
   - Support three-level precedence (system/user/local)
   - Validate merged results
   - Preserve comments and structure when possible

3. **Edge Case Handling:**
   - Quote values containing `=`, `;`, `#` automatically
   - Validate section/key names
   - Handle empty values
   - Support multiline values with indentation continuation (optional)
   - Detect duplicate sections and merge them

4. **Validation:**
   - Strict mode: detect and report duplicate keys
   - Report invalid section/key names
   - Verify INI structure integrity after merge
   - Warn about reserved words in values

5. **Type System:**
   - Treat all values as strings by default (safest approach)
   - Optional type inference for common types (boolean, numeric)
   - Configurable parser behavior for different INI dialects

---

*Document compiled from multiple authoritative sources: Wikipedia, FileFormat.com, official Git documentation, EditorConfig specification, PHP/Python documentation, and GitHub discussions from major INI parser projects.*
