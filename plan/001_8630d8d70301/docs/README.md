# CLI Design Research Documentation

## Overview

This directory contains comprehensive research on CLI design best practices from leading tools (Git, Docker, Cargo, Kubectl). The research is organized to be actionable for the Jin debugger CLI implementation.

**Research Completed:** December 27, 2025
**Scope:** Command naming, subcommand organization, flags, help text, error messages

---

## Files in This Directory

### 1. **SUMMARY.md** (Quick Reference)
**318 lines** - Start here for quick lookup
- Tabular summaries of all key practices
- Quick checklists
- Real-world tool pattern comparison
- Direct examples you can reference

**Use this for:**
- Quick reference while implementing
- Showing team guidelines
- Checking if a pattern is correct
- Making design decisions

### 2. **cli_design_patterns.md** (Comprehensive Guide)
**922 lines** - Detailed reference with full context
- Detailed explanations of each principle
- Extended examples from real tools
- Best practice reasoning and justifications
- Error patterns and edge cases
- Complete citations and source URLs

**Use this for:**
- Understanding the "why" behind practices
- Learning from real-world examples
- Discovering advanced patterns
- Research and deep dives
- Justifying design decisions to team members

---

## Key Topics Covered

### 1. Command Naming Conventions
- Clarity over brevity principle
- Verb-noun vs noun-verb patterns
- Real-world examples from Git, Docker, Cargo, Kubectl
- Case sensitivity and character rules

### 2. Subcommand Organization
- Category-based organization (Cargo style)
- Resource-based organization (Docker style)
- Verb-based organization (Kubectl style)
- Pattern selection guidance

### 3. Flag Naming & Consistency
- Standard reserved flags (-h, -v, -q, -f, -a, -n, -m)
- Kebab-case enforcement
- Short and long form pairing
- Related flag grouping with prefixes
- Flags vs arguments reasoning

### 4. Help Text Structure
- Three-level help system (brief, full, deep dive)
- Help text best practices
- Example-driven documentation
- Next steps and guidance
- Skim-friendly formatting

### 5. Error Message Patterns
- Five error categories with examples
- Actionable error messages
- Exit codes and automation
- Error message best practices
- Tool-specific error patterns

---

## Quick Navigation

### By Question

**Q: "Should I use verb-noun or noun-verb?"**
→ See SUMMARY.md "COMMAND NAMING CONVENTIONS" or cli_design_patterns.md Section 1

**Q: "How should I organize my subcommands?"**
→ See SUMMARY.md "SUBCOMMAND ORGANIZATION" or cli_design_patterns.md Section 2

**Q: "What flags should I reserve and how should I name them?"**
→ See SUMMARY.md "FLAG NAMING & CONSISTENCY" or cli_design_patterns.md Section 3

**Q: "What should my help output look like?"**
→ See SUMMARY.md "HELP TEXT STRUCTURE" or cli_design_patterns.md Section 4

**Q: "How should I format error messages?"**
→ See SUMMARY.md "ERROR MESSAGE PATTERNS" or cli_design_patterns.md Section 5

### By Tool

**Git examples:**
- See cli_design_patterns.md: "Real-World Examples" under Section 1
- See cli_design_patterns.md: "Error Examples" under Section 5

**Docker examples:**
- See cli_design_patterns.md: "Real-World Examples" under Section 1
- See SUMMARY.md table showing noun-verb pattern
- See cli_design_patterns.md Section 2: "Pattern 2: Resource-Based Organization"

**Cargo examples:**
- See cli_design_patterns.md: "Real-World Examples" under Section 1
- See cli_design_patterns.md Section 2: "Pattern 1: Category-Based Organization"

**Kubectl examples:**
- See cli_design_patterns.md: "Real-World Examples" under Section 1
- See cli_design_patterns.md Section 2: "Pattern 3: Verb-Based Organization"

### By Implementation Stage

**Planning phase:**
- SUMMARY.md "Quick Checklist for Jin CLI Design"
- SUMMARY.md "Quick Findings by Topic"

**Development phase:**
- SUMMARY.md "Recommended Tools for Implementation"
- cli_design_patterns.md Section 3 for flag implementation details
- cli_design_patterns.md Section 4 for help text structure

**Testing phase:**
- cli_design_patterns.md Section 4: Help text best practices
- cli_design_patterns.md Section 5: Error message patterns
- SUMMARY.md "Quick Checklist" for verification

---

## Key Takeaways for Jin

1. **Pick a pattern and be consistent**
   - Recommended: Verb-noun (like Git, Cargo)
   - Alternative: Noun-verb (like Docker)
   - Don't mix patterns

2. **Organize into logical categories**
   - 4-6 categories maximum (don't overcomplicate)
   - Suggested: Build, Debug, Inspect, Configuration

3. **Reserve standard flags**
   - Always provide -h/--help, -v/--verbose, etc.
   - Use kebab-case exclusively

4. **Implement three-level help**
   - Brief: Quick understanding (3-5 sec)
   - Full: Complete reference
   - Deep: Extended documentation

5. **Make errors actionable**
   - Explain what went wrong
   - Suggest how to fix it
   - Point to next steps

---

## Research Sources

All sources are cited within the documents with active hyperlinks:

**Official Guides:**
- Command Line Interface Guidelines (clig.dev)
- Heroku CLI Style Guide
- Better CLI Design Guidelines

**Tool Documentation:**
- Git Architecture (Architecture of Open Source Applications)
- Cargo Book
- Docker Container Reference
- Kubernetes Kubectl Reference

**Specialized Resources:**
- 10 Design Principles for Delightful CLIs (Atlassian)
- CLI Design Best Practices (Cody A. Ray)
- Rust Clap Framework Documentation
- CLI UX Best Practices (Evil Martians)
- CLI Design Guidelines (Thoughtworks)

Total sources: 12+ authoritative references

---

## How to Use These Documents

### For CLI Design Decisions
1. Check SUMMARY.md first for quick answer
2. If you need more detail, go to cli_design_patterns.md
3. Use cited sources for additional context

### For Team Communication
- Share SUMMARY.md as design guidelines
- Reference specific sections when reviewing code/help text
- Use examples from cli_design_patterns.md to justify decisions

### For Implementation
- Print the quick checklist from SUMMARY.md
- Keep cli_design_patterns.md open while building
- Refer to tool-specific examples as needed

### For Training/Onboarding
- Start new developers with SUMMARY.md
- Direct them to cli_design_patterns.md for deeper understanding
- Use as reference during code review

---

## Document Metadata

- **Created:** December 27, 2025
- **Last Updated:** December 27, 2025
- **Total Size:** 40KB
- **Total Lines:** 1,240+ (across all files)
- **Status:** Complete and ready for implementation
- **Location:** `/home/dustin/projects/jin/plan/P4M1T1/research/`

---

## Next Steps

These documents are ready for use in Jin CLI implementation. Recommendations:

1. **Review as team** - Schedule 30-minute review of SUMMARY.md
2. **Create design doc** - Use these guidelines to design Jin's command structure
3. **Implement systematically** - Start with command structure, then flags, then help text
4. **Reference during code review** - Use SUMMARY.md as a checklist
5. **Evolve as needed** - Update these docs as you discover patterns in your implementation

---

## Questions?

If you need to extend this research:
- **Rust Clap specific:** See references to Rust Clap documentation
- **Tool comparisons:** See "Real-World Tool Patterns" section
- **Edge cases:** See detailed examples in cli_design_patterns.md
- **Error handling:** See "Error Message Patterns" section
