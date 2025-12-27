# INI File Format - Code Examples and Implementations

## JavaScript/Node.js Parser Example

### Basic Parser Implementation

```javascript
class INIParser {
  constructor(options = {}) {
    this.options = {
      trimValues: true,
      allowDuplicateKeys: false,
      allowDuplicateSections: true,
      commentCharacters: [';', '#'],
      caseInsensitive: false,
      strictMode: true,
      ...options
    };
  }

  parse(content) {
    const lines = content.split(/\r?\n/);
    const result = {};
    let currentSection = null;

    for (let i = 0; i < lines.length; i++) {
      let line = lines[i];

      // Remove comments
      line = this.removeComments(line);
      line = line.trim();

      // Skip empty lines
      if (!line) continue;

      // Handle section headers
      if (line.startsWith('[') && line.endsWith(']')) {
        currentSection = this.parseSection(line);

        if (!result[currentSection]) {
          result[currentSection] = {};
        } else if (!this.options.allowDuplicateSections) {
          throw new Error(`Duplicate section: ${currentSection}`);
        }
      }
      // Handle key-value pairs
      else if (line.includes('=') || line.includes(':')) {
        if (!currentSection) {
          throw new Error(`Key-value pair outside section at line ${i + 1}`);
        }

        const [key, value] = this.parseKeyValue(line);

        if (result[currentSection][key] && !this.options.allowDuplicateKeys) {
          if (this.options.strictMode) {
            throw new Error(`Duplicate key '${key}' in section [${currentSection}]`);
          }
        }

        result[currentSection][key] = this.options.trimValues ?
          value.trim() : value;
      }
      else if (this.options.strictMode) {
        throw new Error(`Invalid line at ${i + 1}: ${line}`);
      }
    }

    return result;
  }

  parseSection(line) {
    // Handle Git config subsections: [section "subsection"]
    const match = line.match(/^\[([^\]"]+)(?:\s"([^"]+)")?\]$/);

    if (match) {
      if (match[2]) {
        // Subsection syntax: section.subsection
        return `${match[1]}.${match[2]}`;
      }
      return match[1];
    }

    throw new Error(`Invalid section header: ${line}`);
  }

  parseKeyValue(line) {
    // Use first occurrence of = or :
    const delimiterIndex = Math.min(
      line.indexOf('=') >= 0 ? line.indexOf('=') : Infinity,
      line.indexOf(':') >= 0 ? line.indexOf(':') : Infinity
    );

    if (delimiterIndex === Infinity) {
      throw new Error(`No delimiter in: ${line}`);
    }

    const key = line.substring(0, delimiterIndex).trim();
    const value = line.substring(delimiterIndex + 1);

    // Handle quoted values
    let unquotedValue = value.trim();
    if ((unquotedValue.startsWith('"') && unquotedValue.endsWith('"')) ||
        (unquotedValue.startsWith("'") && unquotedValue.endsWith("'"))) {
      unquotedValue = unquotedValue.slice(1, -1);
    }

    return [key, unquotedValue];
  }

  removeComments(line) {
    for (const char of this.options.commentCharacters) {
      const commentIndex = line.indexOf(char);
      if (commentIndex !== -1) {
        // Check if comment is in a quoted value
        const beforeComment = line.substring(0, commentIndex);
        const quoteCount = (beforeComment.match(/"/g) || []).length;

        // If even number of quotes, the comment char is not quoted
        if (quoteCount % 2 === 0) {
          return line.substring(0, commentIndex);
        }
      }
    }
    return line;
  }

  stringify(obj) {
    const lines = [];

    for (const [section, values] of Object.entries(obj)) {
      lines.push(`[${section}]`);

      for (const [key, value] of Object.entries(values)) {
        const quotedValue = this.shouldQuoteValue(value) ?
          `"${value}"` : value;
        lines.push(`  ${key} = ${quotedValue}`);
      }

      lines.push('');
    }

    return lines.join('\n');
  }

  shouldQuoteValue(value) {
    // Quote if contains special characters
    return /[=;:#\n\t]/.test(value) ||
           /^\s|\s$/.test(value) || // Leading/trailing whitespace
           ['true', 'false', 'null', 'yes', 'no', 'on', 'off'].includes(value);
  }
}

// Usage
const iniContent = `
; Database Configuration
[database]
host = localhost
port = 3306
name = myapp_db

[database.connection]
timeout = 30
retry_count = 3
`;

const parser = new INIParser({ strictMode: true });
const config = parser.parse(iniContent);
console.log(JSON.stringify(config, null, 2));

// Output:
// {
//   "database": {
//     "host": "localhost",
//     "port": "3306",
//     "name": "myapp_db"
//   },
//   "database.connection": {
//     "timeout": "30",
//     "retry_count": "3"
//   }
// }
```

### Merging Implementation

```javascript
class INIMerger {
  static merge(baseConfig, overrideConfig) {
    const result = { ...baseConfig };

    for (const section in overrideConfig) {
      if (result[section]) {
        // Merge keys in existing section
        result[section] = {
          ...result[section],
          ...overrideConfig[section]
        };
      } else {
        // Add new section
        result[section] = { ...overrideConfig[section] };
      }
    }

    return result;
  }

  static deepMerge(baseConfig, overrideConfig) {
    const result = JSON.parse(JSON.stringify(baseConfig));

    const mergeObjects = (base, override) => {
      for (const key in override) {
        if (typeof override[key] === 'object' && override[key] !== null) {
          if (typeof base[key] === 'object' && base[key] !== null) {
            mergeObjects(base[key], override[key]);
          } else {
            base[key] = override[key];
          }
        } else {
          base[key] = override[key];
        }
      }
    };

    mergeObjects(result, overrideConfig);
    return result;
  }

  static mergeMultiple(...configs) {
    return configs.reduce((merged, config) =>
      this.merge(merged, config), {}
    );
  }

  static mergeFiles(basePath, overridePath, parser) {
    const baseContent = require('fs').readFileSync(basePath, 'utf-8');
    const overrideContent = require('fs').readFileSync(overridePath, 'utf-8');

    const baseConfig = parser.parse(baseContent);
    const overrideConfig = parser.parse(overrideContent);

    return this.merge(baseConfig, overrideConfig);
  }
}

// Example usage
const baseConfig = {
  'app': { name: 'MyApp', debug: 'false' },
  'database': { host: 'localhost', port: '3306' }
};

const devConfig = {
  'app': { debug: 'true' },
  'database': { database: 'myapp_dev' }
};

const merged = INIMerger.merge(baseConfig, devConfig);
// Result:
// {
//   'app': { name: 'MyApp', debug: 'true' },
//   'database': { host: 'localhost', port: '3306', database: 'myapp_dev' }
// }
```

## Real-World Examples

### Git Config Example

```ini
[core]
    repositoryformatversion = 0
    filemode = true
    bare = false
    logallrefupdates = true
    ignorecase = true

[user]
    name = John Doe
    email = john.doe@example.com

[url "git@github.com:"]
    insteadOf = https://github.com/

[remote "origin"]
    url = https://github.com/user/project.git
    fetch = +refs/heads/*:refs/remotes/origin/*

[remote "upstream"]
    url = https://github.com/original/project.git
    fetch = +refs/heads/*:refs/remotes/upstream/*

[branch "main"]
    remote = origin
    merge = refs/heads/main

[alias]
    st = status
    co = checkout
    br = branch -v
    ci = commit
    unstage = reset HEAD --
    last = log -1 HEAD
    visual = log --graph --oneline --decorate --all

[color]
    ui = true
    diff = true
    status = true

[diff]
    tool = vimdiff

[merge]
    tool = vimdiff
    conflictstyle = diff3

[init]
    defaultBranch = main
```

**Parsed as:**
```javascript
{
  "core": {
    "repositoryformatversion": "0",
    "filemode": "true",
    "bare": "false",
    "logallrefupdates": "true",
    "ignorecase": "true"
  },
  "user": {
    "name": "John Doe",
    "email": "john.doe@example.com"
  },
  "url.git@github.com": {
    "insteadOf": "https://github.com/"
  },
  "remote.origin": {
    "url": "https://github.com/user/project.git",
    "fetch": "+refs/heads/*:refs/remotes/origin/*"
  },
  "remote.upstream": {
    "url": "https://github.com/original/project.git",
    "fetch": "+refs/heads/*:refs/remotes/upstream/*"
  },
  "branch.main": {
    "remote": "origin",
    "merge": "refs/heads/main"
  },
  "alias": {
    "st": "status",
    "co": "checkout",
    "br": "branch -v",
    "ci": "commit",
    "unstage": "reset HEAD --",
    "last": "log -1 HEAD",
    "visual": "log --graph --oneline --decorate --all"
  },
  "color": {
    "ui": "true",
    "diff": "true",
    "status": "true"
  },
  "diff": {
    "tool": "vimdiff"
  },
  "merge": {
    "tool": "vimdiff",
    "conflictstyle": "diff3"
  },
  "init": {
    "defaultBranch": "main"
  }
}
```

### EditorConfig Example

```ini
root = true

[*]
indent_style = space
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.md]
trim_trailing_whitespace = false

[*.{js,jsx,ts,tsx}]
indent_size = 2

[*.{py,pyx,pyi}]
indent_size = 4

[*.go]
indent_style = tab

[Makefile]
indent_style = tab

[*.{json,yaml,yml}]
indent_size = 2

[*.{html,xml}]
indent_size = 2
```

### Application Config Example

```ini
; Default configuration
[app]
name = MyApplication
version = 1.0.0
environment = development
debug = false

[server]
host = 0.0.0.0
port = 8080
workers = 4
timeout = 30

[database]
driver = postgresql
host = localhost
port = 5432
database = myapp
username = app_user
pool_size = 10

[logging]
level = INFO
format = json
output = stdout

[cache]
enabled = true
ttl = 3600
driver = redis
host = localhost
port = 6379

[features]
auth_enabled = true
api_v2 = true
experimental_feature = false
```

**Development Override:**
```ini
; Development overrides
[app]
debug = true
environment = development

[server]
port = 3000
workers = 1

[logging]
level = DEBUG
output = stdout

[cache]
enabled = false
```

**Production Override:**
```ini
; Production overrides
[app]
debug = false
environment = production

[server]
host = 10.0.0.1
port = 8080
workers = 8
timeout = 60

[database]
host = db.prod.internal
port = 5432
pool_size = 20

[logging]
level = WARN
output = /var/log/app.log

[cache]
enabled = true
ttl = 7200
host = cache.prod.internal
```

## Testing Examples

### Test Cases for Parser

```javascript
const testCases = [
  {
    name: "Basic section and key-value",
    input: "[section]\nkey = value",
    expected: { "section": { "key": "value" } }
  },
  {
    name: "Multiple sections",
    input: "[section1]\nkey1 = value1\n[section2]\nkey2 = value2",
    expected: {
      "section1": { "key1": "value1" },
      "section2": { "key2": "value2" }
    }
  },
  {
    name: "Quoted value with equals",
    input: '[section]\nkey = "value=with=equals"',
    expected: { "section": { "key": "value=with=equals" } }
  },
  {
    name: "Comments ignored",
    input: "[section]\n; comment\nkey = value # inline comment",
    expected: { "section": { "key": "value" } },
    strict: false
  },
  {
    name: "Empty value",
    input: "[section]\nkey =",
    expected: { "section": { "key": "" } }
  },
  {
    name: "Colon delimiter",
    input: "[section]\nkey : value",
    expected: { "section": { "key": "value" } }
  },
  {
    name: "Whitespace trimming",
    input: "[section]\nkey   =   value   ",
    expected: { "section": { "key": "value" } }
  },
  {
    name: "Git config subsection",
    input: '[remote "origin"]\nurl = https://github.com/user/repo.git',
    expected: { "remote.origin": { "url": "https://github.com/user/repo.git" } }
  }
];
```

### Test Cases for Merge

```javascript
const mergeTests = [
  {
    name: "Simple override",
    base: { "app": { "debug": "false" } },
    override: { "app": { "debug": "true" } },
    expected: { "app": { "debug": "true" } }
  },
  {
    name: "New section added",
    base: { "app": { "name": "App" } },
    override: { "server": { "port": "8080" } },
    expected: {
      "app": { "name": "App" },
      "server": { "port": "8080" }
    }
  },
  {
    name: "New key in existing section",
    base: { "app": { "name": "App" } },
    override: { "app": { "debug": "true" } },
    expected: {
      "app": { "name": "App", "debug": "true" }
    }
  },
  {
    name: "Preserve unmodified keys",
    base: { "db": { "host": "localhost", "port": "3306" } },
    override: { "db": { "port": "5432" } },
    expected: {
      "db": { "host": "localhost", "port": "5432" }
    }
  }
];
```

## Edge Case Handling

### Handling Equals Signs in Values

```javascript
// Test case
const input = `
[settings]
; Properly quoted value with equals
connection_string = "Server=localhost;Port=3306;User=admin"
equation = "2+2=4"
`;

const parser = new INIParser();
const result = parser.parse(input);
// Result:
// {
//   "settings": {
//     "connection_string": "Server=localhost;Port=3306;User=admin",
//     "equation": "2+2=4"
//   }
// }
```

### Handling Duplicate Keys

```javascript
// Option 1: First-value-wins
const parser1 = new INIParser({
  allowDuplicateKeys: false,
  strictMode: false,
  duplicateKeyBehavior: 'preserve'
});

// Option 2: Last-value-wins (default)
const parser2 = new INIParser({
  allowDuplicateKeys: true,
  duplicateKeyBehavior: 'override'
});

// Option 3: Strict mode (throw error)
const parser3 = new INIParser({
  allowDuplicateKeys: false,
  strictMode: true
});
```

### Handling Duplicate Sections

```javascript
const input = `
[database]
host = localhost
port = 3306

[other]
timeout = 30

[database]
username = admin
password = secret
`;

const parser = new INIParser({ allowDuplicateSections: true });
const result = parser.parse(input);
// Merges duplicate sections:
// {
//   "database": {
//     "host": "localhost",
//     "port": "3306",
//     "username": "admin",
//     "password": "secret"
//   },
//   "other": {
//     "timeout": "30"
//   }
// }
```

---

*Code examples demonstrate practical INI parsing and merging implementations suitable for the P2M2 MergeValue system.*
