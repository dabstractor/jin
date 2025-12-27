# Array Merge Strategies Research - Complete Index

## Overview

This is a comprehensive research project on array merge strategies in configuration file merging systems. The research covers theoretical foundations, real-world tool implementations, practical code examples, and best practices.

**Total Research Size:** 2,572 lines across 4 markdown documents

## Documents

### 1. ARRAY_MERGE_SUMMARY.md (199 lines) - START HERE
**Quick reference and key takeaways**

- The 5 core array merge strategies at a glance
- One-paragraph overview of each real-world tool
- Quick comparison table
- Best practices checklist
- Next steps for jin project

**Ideal for:** Quick reference, executive summary, first-time readers

---

### 2. ARRAY_MERGE_STRATEGIES.md (1,360 lines) - COMPREHENSIVE REFERENCE
**Complete research documentation**

**Sections:**

1. **Common Array Merge Strategies** (Replace, Append, Prepend, Union, Keyed)
   - Detailed explanation of each strategy
   - Characteristics and use cases
   - Code examples for each

2. **Real-World Tool Implementations**
   - Kubernetes Strategic Merge Patch
     - Patch merge keys, directives, backward compatibility
   - webpack-merge customizeArray
     - Built-in strategies, custom functions, wildcard patterns
   - Helm Chart Value Merging
     - Default behavior, limitations, workarounds
   - Docker Compose Override Merging
     - Append behavior, replace behavior, edge cases
   - ESLint Config Merging
     - Extends array, rule merging, override blocks

3. **Keyed Array Merging Patterns**
   - Identifying array items (ID vs name fields)
   - Handling missing keys
   - Handling duplicate keys
   - Ordering after merge
   - Deletion markers

4. **Configuration Patterns**
   - Per-field strategy annotation
   - Global default strategy
   - Schema-driven merge rules

5. **Edge Cases**
   - Empty arrays
   - Null elements in arrays
   - Mixed types in arrays

6. **Rust Implementation Examples**
   - Keyed array merge (jin project)
   - Multi-format MergeValue type
   - Using merge crate with derive macros
   - Null deletion pattern

7. **References**
   - Official documentation links
   - Rust libraries and crates
   - Related tools and research sources

**Ideal for:** Deep understanding, implementation reference, research

---

### 3. ARRAY_MERGE_CODE_EXAMPLES.md (1,013 lines) - PRACTICAL EXAMPLES
**Copy-paste-ready code snippets**

**Sections:**

1. **JavaScript Examples**
   - Replace strategy (simple)
   - Append strategy
   - Union strategy (deduplicate)
   - Deep merge with strategies
   - Keyed array merge (by object id)
   - webpack-merge style

2. **Rust Examples**
   - Simple replace strategy
   - Append strategy
   - Union strategy (deduplication)
   - Keyed merge (by ID)
   - Using merge crate with derive
   - Generic deep merge
   - jin-style keyed merge (actual project code)

3. **YAML Configuration Examples**
   - Simple append (default)
   - Keyed merge (Kubernetes style)
   - Null deletion
   - Map-based merge (alternative)
   - Separate base and override lists

4. **JSON Schema Examples**
   - Schema with merge keys
   - Per-field strategies
   - Custom merge key

5. **Common Patterns**
   - Environment-specific configuration
   - Layer merging (multiple files)
   - Validated merge
   - Audit trail (tracking changes)
   - Conditional merge

6. **Testing Examples**
   - JavaScript unit tests
   - Rust unit tests

**Ideal for:** Implementation, learning by example, testing

---

## Research Coverage

### Array Merge Strategies Covered

| Strategy | Depth | Example Tools | Code Examples |
|----------|-------|----------------|----------------|
| **Replace** | Comprehensive | Kubernetes, Docker, Helm | JS, Rust, YAML, JSON |
| **Append** | Comprehensive | webpack-merge, Docker | JS, Rust, YAML |
| **Prepend** | Moderate | webpack-merge, ESLint | JS, Rust |
| **Union** | Moderate | webpack-merge | JS, Rust |
| **Keyed Merge** | Deep | Kubernetes, jin | JS, Rust (detailed) |

### Tools Researched

| Tool | Research Depth | Key Findings | References |
|------|----------------|--------------|-----------|
| **Kubernetes** | Deep | Strategic merge patch, patch merge keys, directives | 3 references |
| **webpack-merge** | Deep | Custom strategies, wildcards, unique filtering | 2 references |
| **Helm** | Moderate | Array limitation, workarounds | 2 references |
| **Docker Compose** | Moderate | Inconsistent array merging | 2 references |
| **ESLint** | Moderate | Rule merging, extends, overrides | 2 references |

### Rust Libraries Covered

| Crate | Status | Examples | Use Case |
|-------|--------|----------|----------|
| **merge** | Documented | Yes | Derive-based strategies |
| **serde-toml-merge** | Documented | Mentioned | TOML-specific |
| **deepmerge** | Documented | Mentioned | Flexible merging |
| **schematic** | Documented | Mentioned | Full config system |
| **config** | Documented | Mentioned | Hierarchical config |

### Edge Cases Covered

- Empty arrays (Lodash bug discussed)
- Null elements in arrays (format-specific)
- Mixed types in arrays (homogeneity)
- Missing keys in keyed arrays (fallback behavior)
- Duplicate keys (last-wins vs first-wins)
- Array ordering (preservation vs sorting)

---

## How to Use This Research

### For Understanding Concepts
1. Start with **SUMMARY.md** for quick orientation
2. Read relevant section in **STRATEGIES.md** for deep dive
3. Reference **CODE_EXAMPLES.md** for implementation ideas

### For Implementation
1. Choose merge strategy from SUMMARY table
2. Find implementation pattern in CODE_EXAMPLES
3. Consult STRATEGIES for edge cases
4. Check references for official documentation

### For Tool-Specific Research
Look up tool name in STRATEGIES.md:
- **Kubernetes** → Strategic Merge Patch section
- **webpack-merge** → customizeArray section
- **Helm** → Value Merging section
- **Docker Compose** → Override Merging section
- **ESLint** → Config Merging section

### For Rust-Specific Information
- **derive macros** → merge crate examples in CODE_EXAMPLES
- **generic implementation** → deep_merge examples in CODE_EXAMPLES
- **serde integration** → MergeValue type in STRATEGIES
- **project reference** → jin implementation in STRATEGIES and CODE_EXAMPLES

---

## Key Findings Summary

### Most Important Insights

1. **No Universal Strategy**
   - Different tools use different defaults
   - Context matters (objects vs arrays vs scalars)
   - Explicit specification beats assumptions

2. **Keyed Merge Complexity**
   - Essential for object arrays
   - Requires careful handling of missing/duplicate keys
   - Falls back gracefully to replace strategy
   - jin project uses conservative approach (good design)

3. **Format Constraints Matter**
   - TOML lacks null support
   - INI limited nesting
   - YAML most flexible
   - Affects merge strategy choices

4. **The Array Merge Problem**
   - Helm acknowledges it as a "known limitation"
   - Docker Compose has inconsistent behavior
   - Kubernetes solved with patch merge keys
   - No obvious "right answer"

5. **Best Practice: Be Explicit**
   - Document merge behavior
   - Use schema annotations
   - Provide per-field configuration
   - Validate after merge

### Common Patterns Across Tools

- Objects always deep merge
- Arrays need explicit strategy
- Null deletes keys (in most systems)
- Overlay/later config has precedence
- Primitives are replaced (not merged)

---

## References Overview

### Official Documentation (11 sources)
- Kubernetes community docs
- webpack-merge GitHub and npm
- Helm official guide
- Docker Compose official docs
- ESLint official docs

### Rust Ecosystem (5 sources)
- merge crate docs
- serde-toml-merge GitHub
- deepmerge docs
- schematic docs
- Cargo documentation

### Research Sources (10+ sources)
- PHP documentation
- JavaScript research
- YAML tools
- JSON schema tools
- GitHub issues and discussions

---

## Quick Links to Key Content

### By Strategy
- **Replace:** SUMMARY (table), STRATEGIES (section 1.1), CODE_EXAMPLES (JS ex 1, Rust ex 1)
- **Append:** SUMMARY (table), STRATEGIES (section 1.2), CODE_EXAMPLES (JS ex 2, Rust ex 2, YAML ex 1)
- **Prepend:** STRATEGIES (section 1.3), CODE_EXAMPLES (JS ex 6 with webpack)
- **Union:** STRATEGIES (section 1.4), CODE_EXAMPLES (JS ex 3, Rust ex 3)
- **Keyed:** SUMMARY, STRATEGIES (section 3), CODE_EXAMPLES (JS ex 5, Rust ex 4 & 7)

### By Tool
- **Kubernetes:** STRATEGIES (section 2.1), CODE_EXAMPLES (YAML ex 2)
- **webpack:** STRATEGIES (section 2.2), CODE_EXAMPLES (JS ex 6, pattern section)
- **Helm:** STRATEGIES (section 2.3), CODE_EXAMPLES (YAML ex 4-5)
- **Docker:** STRATEGIES (section 2.4), CODE_EXAMPLES (YAML ex 1)
- **ESLint:** STRATEGIES (section 2.5)

### By Topic
- **Edge Cases:** STRATEGIES (section 5), CODE_EXAMPLES (test section)
- **Schema:** STRATEGIES (section 4.3), CODE_EXAMPLES (JSON schema section)
- **Testing:** CODE_EXAMPLES (test examples section)
- **Rust:** STRATEGIES (section 6), CODE_EXAMPLES (Rust section)

---

## Integration with jin Project

The jin project at `/home/dustin/projects/jin` implements a sophisticated merge system:

**Current Implementation:**
- Located in `/src/merge/deep.rs` and `/src/merge/value.rs`
- Uses keyed merge (id/name fields) for arrays
- Falls back to replace if keys missing
- Supports JSON, YAML, TOML, INI formats
- Uses IndexMap to preserve order

**Key Code:**
```rust
// From /home/dustin/projects/jin/src/merge/deep.rs
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue>
fn merge_arrays(base: Vec<MergeValue>, overlay: Vec<MergeValue>) -> Result<Vec<MergeValue>>
fn extract_array_keys(arr: &[MergeValue]) -> Option<IndexMap<String, MergeValue>>
```

**Design Choices (well-made):**
1. Conservative keyed merge (requires all items have key)
2. Fallback to replace when keys missing
3. Null-based deletion for keys
4. Recursive deep merge for matched keys
5. Order preservation via IndexMap

---

## How to Continue This Research

### Future Enhancements
1. **Schema Validation**
   - Add JSON Schema validation before/after merge
   - Document schema requirements per tool

2. **Performance Analysis**
   - Benchmark different merge strategies
   - Memory usage comparison
   - Large dataset handling

3. **Additional Tools**
   - Kustomize in detail
   - Pulumi merge behavior
   - Terraform override merging
   - AWS CloudFormation parameter merging

4. **Advanced Patterns**
   - Conflict resolution strategies
   - Merge strategies in streams
   - Incremental merging
   - Distributed config merging

5. **Visual Documentation**
   - Merge strategy flowcharts
   - Tool comparison matrices
   - Example transformation diagrams

---

## Document Statistics

- **Total Lines:** 2,572
- **Total Words:** ~15,000
- **Code Examples:** 50+
- **Tools Covered:** 5 major + 10+ supporting
- **Rust Examples:** 15+
- **Test Cases:** 12+ examples
- **Schema Examples:** 4+
- **References:** 25+

---

## Citation Information

This research documents:
- **Published tools & frameworks** with their official documentation
- **Code patterns** from open-source projects
- **Best practices** from industry-standard tools
- **Practical examples** from real-world implementations

All references to external tools link to their official documentation or GitHub repositories.

---

## Closing Notes

Array merging is a deceptively complex problem in configuration management. This research shows that:

1. **No one-size-fits-all solution** exists
2. **Context matters** - object arrays need different handling than primitive arrays
3. **Explicit is better than implicit** - document and specify merge behavior
4. **Real-world tools vary** - understand your tool's specific behavior
5. **Edge cases matter** - empty arrays, missing keys, null handling

The jin project's implementation represents a well-thought-out approach that balances practicality with robustness.

---

## Document Map

```
ARRAY_MERGE_RESEARCH_INDEX.md (this file)
├── ARRAY_MERGE_SUMMARY.md (2 min read)
├── ARRAY_MERGE_STRATEGIES.md (15 min read)
├── ARRAY_MERGE_CODE_EXAMPLES.md (20 min read)
└── Cross-referenced throughout
```

Start with SUMMARY.md, then use the Quick Links section to find what you need.

---

**Last Updated:** December 27, 2025
**Research Scope:** Configuration file merging, array merge strategies, real-world implementations
**Focus:** Practical information with code examples and references
