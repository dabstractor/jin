# Multi-Layer Configuration Merge Orchestration Research

## Executive Summary

This research documents patterns and algorithms for multi-layer configuration merge orchestration, with focus on precedence-based merging, tree walking patterns, and RFC 7396 JSON Merge Patch implementations. The findings provide practical guidance for building deterministic configuration systems.

## 1. Layer Merge Orchestration Patterns

### 1.1 Precedence-Based Layer Merging

#### Common Precedence Models

1. **Hierarchical Override Model**
   - Lower levels override higher levels
   - Pattern: `defaults < environment < user < command-line`
   - Used by: Spring Boot, Docker Compose, Kubernetes

2. **Stack-Based Model**
   - Later layers override earlier layers
   - Pattern: `base → dev → prod → overrides`
   - Used by: Helm Charts, Terraform

3. **Contextual Precedence**
   - Precedence depends on configuration context
   - Pattern: `resource-specific > template > global`

#### Algorithm Implementation

```typescript
interface MergeLayer {
  id: string;
  priority: number;
  config: Record<string, any>;
  mergeStrategy?: 'deep' | 'shallow' | 'array-merge';
}

class LayeredMerger {
  private layers: MergeLayer[] = [];

  addLayer(layer: MergeLayer) {
    this.layers.push(layer);
    this.layers.sort((a, b) => b.priority - a.priority); // Higher priority = higher precedence
  }

  merge(): Record<string, any> {
    const result: Record<string, any> = {};

    for (const layer of this.layers) {
      this.applyLayer(result, layer.config, layer.mergeStrategy || 'deep');
    }

    return result;
  }

  private applyLayer(target: Record<string, any>, source: Record<string, any>, strategy: string) {
    for (const [key, value] of Object.entries(source)) {
      if (value === null) {
        delete target[key];
      } else if (typeof value === 'object' && !Array.isArray(value)) {
        if (!target[key] || typeof target[key] !== 'object') {
          target[key] = {};
        }
        this.applyLayer(target[key], value, strategy);
      } else {
        target[key] = value;
      }
    }
  }
}
```

## 2. Tree Walking Patterns for Configuration Merging

### 2.1 Post-Order Traversal for Merging

Post-order traversal is optimal for tree-based merging because it processes children before parents, allowing proper inheritance and override resolution.

```python
class TreeNode:
    def __init__(self, key, value=None, children=None):
        self.key = key
        self.value = value
        self.children = children or []

    def merge_post_order(self, other_node):
        """Merge another node using post-order traversal"""
        if not other_node:
            return self

        # First merge children recursively
        for child in other_node.children:
            matching_child = self.find_child(child.key)
            if matching_child:
                matching_child.merge_post_order(child)
            else:
                self.children.append(child)

        # Then merge current node (children processed first)
        if other_node.value is not None:
            self.value = other_node.value

        return self

    def find_child(self, key):
        return next((child for child in self.children if child.key == key), None)

# Usage example
def build_config_tree(config: dict, parent_key=""):
    """Build tree structure from nested config"""
    if not config:
        return None

    root = TreeNode(parent_key)
    for key, value in config.items():
        if isinstance(value, dict):
            child = build_config_tree(value, key)
            if child:
                root.children.append(child)
        else:
            root.children.append(TreeNode(key, value))

    return root
```

### 2.2 Git Tree Walking Patterns

Git's merge algorithm uses tree walking with the following patterns:

1. **Three-way merge**: Find common ancestor, compare with both parents
2. **Blob walking**: Compare content at different tree levels
3. **Directory-level merging**: Merge entire directories then merge files within

```python
class GitTreeWalker:
    def __init__(self, repo):
        self.repo = repo

    def merge_trees(self, base_tree, ours_tree, theirs_tree):
        """Three-way merge using tree walking"""
        base = self.get_tree(base_tree)
        ours = self.get_tree(ours_tree)
        theirs = self.get_tree(theirs_tree)

        # Find common ancestor
        common_ancestor = self.find_common_ancestor(base, ours, theirs)

        # Merge each path
        all_paths = set(base.keys()) | set(ours.keys()) | set(theirs.keys())

        merged = {}
        for path in all_paths:
            merged[path] = self.merge_path(
                common_ancestor.get(path),
                ours.get(path),
                theirs.get(path)
            )

        return merged
```

## 3. RFC 7396 JSON Merge Patch Patterns

### 3.1 RFC 7396 Implementation

RFC 7396 specifies a JSON Merge Patch format for describing changes to JSON documents.

```javascript
class JSONMergePatch {
  /**
   * Apply a merge patch to a target document
   * @param {Object} target - Target document
   * @param {Object} patch - Merge patch
   * @returns {Object} Merged document
   */
  static apply(target, patch) {
    if (patch === null || typeof patch !== 'object') {
      return patch;
    }

    if (typeof target !== 'object' || target === null) {
      return JSON.parse(JSON.stringify(patch));
    }

    const result = { ...target };

    for (const key in patch) {
      const value = patch[key];

      if (value === null) {
        // Null means delete the key
        delete result[key];
      } else if (typeof value === 'object' && !Array.isArray(value)) {
        // Recursively merge objects
        result[key] = this.apply(result[key] || {}, value);
      } else {
        // Replace the value
        result[key] = value;
      }
    }

    return result;
  }

  /**
   * Create a merge patch from two documents
   * @param {Object} original - Original document
   * @param {Object} modified - Modified document
   * @returns {Object} Merge patch
   */
  static create(original, modified) {
    const patch = {};

    // Check for keys in modified but not in original
    for (const key in modified) {
      if (!(key in original)) {
        patch[key] = modified[key];
      } else if (JSON.stringify(original[key]) !== JSON.stringify(modified[key])) {
        patch[key] = modified[key];
      }
    }

    // Check for keys in original but not in modified
    for (const key in original) {
      if (!(key in modified)) {
        patch[key] = null;
      }
    }

    return patch;
  }
}
```

### 3.2 RFC 7396 Best Practices

1. **Null for deletion**: Use `null` to indicate removal of a key
2. **Object merging**: Nested objects are merged recursively
3. **Array handling**: Arrays are replaced entirely (not merged)
4. **Type safety**: Handle type mismatches appropriately

## 4. Configuration Composition Patterns

### 4.1 Docker Compose Override Pattern

Docker Compose uses a simple but effective layering model:

```yaml
# docker-compose.yml (base)
version: '3.8'
services:
  web:
    image: nginx:latest
    ports:
      - "80:80"
    environment:
      - NODE_ENV=production

# docker-compose.override.yml (development override)
services:
  web:
    volumes:
      - .:/usr/share/nginx/html
    environment:
      - DEBUG=true

# docker-compose.prod.yml (production override)
services:
  web:
    image: nginx:stable
    environment:
      - NODE_ENV=production
      - SENTRY_DSN=${SENTRY_DSN}
```

**Precedence**: `-f file1 -f file2` means file2 overrides file1

### 4.2 Kubernetes ConfigMap Patterns

Kubernetes uses multiple layers of configuration:

```yaml
# base-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-base-config
data:
  config.yaml: |
    database:
      host: localhost
      port: 5432
    features:
      feature1: true
      feature2: false

# override-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-override-config
data:
  config.yaml: |
    database:
      port: 5433
    features:
      feature2: true
      feature3: true
```

**Merging Strategy**: Kubernetes uses strategic merge patch, which intelligently merges objects based on API schema.

### 4.3 Spring Boot Configuration Precedence

Spring Boot follows a strict precedence hierarchy:

1. `@PropertySource` annotations on `@Configuration` classes
2. ServletConfig init parameters
3. ServletContext init parameters
4. Environment variables
5. Externalized properties (application.properties/yml)
6. `@ConfigurationProperties` classes

```java
@Configuration
@PropertySource("classpath:defaults.properties")
@PropertySource("classpath:${environment:dev}/config.properties")
public class AppConfig {

    @Value("${app.timeout:30}")
    private int timeout;

    @Bean
    @ConfigurationProperties(prefix = "database")
    public DatabaseConfig databaseConfig() {
        return new DatabaseConfig();
    }
}
```

## 5. Error Handling Patterns

### 5.1 Merge Conflict Resolution

```typescript
interface MergeConflict {
  path: string;
  leftValue: any;
  rightValue: any;
  leftLayer: string;
  rightLayer: string;
}

class MergeConflictResolver {
  private conflicts: MergeConflict[] = [];

  // Detect conflicts during merge
  detectConflict(target: any, key: string, newValue: any, layerId: string) {
    if (key in target && target[key] !== newValue) {
      this.conflicts.push({
        path: key,
        leftValue: target[key],
        rightValue: newValue,
        leftLayer: 'previous',
        rightLayer: layerId
      });
    }
  }

  // Handle conflicts with different strategies
  resolveConflicts(strategy: 'fail' | 'left-wins' | 'right-wins' | 'merge'): any {
    switch (strategy) {
      case 'fail':
        throw new MergeConflictError(this.conflicts);

      case 'left-wins':
        return this.conflicts.map(c => ({ path: c.path, value: c.leftValue }));

      case 'right-wins':
        return this.conflicts.map(c => ({ path: c.path, value: c.rightValue }));

      case 'merge':
        return this.performSmartMerge();
    }
  }

  private performSmartMerge() {
    // Implement smart merge logic based on data types
    return this.conflicts.map(conflict => {
      if (typeof conflict.leftValue === 'object' &&
          typeof conflict.rightValue === 'object') {
        return {
          path: conflict.path,
          value: { ...conflict.leftValue, ...conflict.rightValue }
        };
      }
      return { path: conflict.path, value: conflict.rightValue };
    });
  }
}
```

### 5.2 Validation Patterns

```typescript
interface ValidationRule {
  path: string;
  validator: (value: any) => boolean;
  message: string;
}

class ConfigurationValidator {
  private rules: ValidationRule[] = [];

  addRule(rule: ValidationRule) {
    this.rules.push(rule);
  }

  validate(config: any): ValidationResult {
    const errors: string[] = [];

    for (const rule of this.rules) {
      const value = this.getNestedValue(config, rule.path);
      if (!rule.validator(value)) {
        errors.push(rule.message);
      }
    }

    return {
      valid: errors.length === 0,
      errors
    };
  }

  private getNestedValue(obj: any, path: string): any {
    return path.split('.').reduce((current, key) => current?.[key], obj);
  }
}

// Usage
const validator = new ConfigurationValidator();
validator.addRule({
  path: 'database.port',
  validator: port => port >= 1024 && port <= 65535,
  message: 'Database port must be between 1024 and 65535'
});
```

## 6. Best Practices for Deterministic Merge Results

### 6.1 Deterministic Ordering

1. **Sort layers by priority**: Always merge in consistent order
2. **Use deterministic data structures**: Maps with sorted keys, ordered arrays
3. **Document precedence**: Clearly document which layer wins conflicts

### 6.2 Idempotent Operations

```python
def idempotent_merge(original, patch):
    """Ensure merge can be applied multiple times with same result"""
    merged = merge(original, patch)
    return merge(merged, patch)  # Second application should have no effect
```

### 6.3 Validation Before Merge

```typescript
interface MergeOperation {
  original: any;
  patches: MergePatch[];
  validate?: (config: any) => void;
}

function safeMerge(operation: MergeOperation): any {
  // Apply all patches
  let result = operation.original;
  for (const patch of operation.patches) {
    result = JSONMergePatch.apply(result, patch);
  }

  // Validate result
  if (operation.validate) {
    operation.validate(result);
  }

  return result;
}
```

### 6.4 Performance Considerations

1. **Lazy evaluation**: Only merge what's needed
2. **Caching**: Cache merged results for unchanged layers
3. **Immutable patterns**: Use immutable data structures to avoid side effects

```typescript
class CachedLayerMerger {
  private cache = new Map<string, any>();

  merge(layers: MergeLayer[]): any {
    const cacheKey = this.createCacheKey(layers);

    if (this.cache.has(cacheKey)) {
      return this.cache.get(cacheKey);
    }

    const result = this.performMerge(layers);
    this.cache.set(cacheKey, result);
    return result;
  }

  private createCacheKey(layers: MergeLayer[]): string {
    return layers
      .sort((a, b) => a.id.localeCompare(b.id))
      .map(layer => `${layer.id}:${layer.priority}:${this.hash(layer.config)}`)
      .join('|');
  }
}
```

## 7. Production Examples

### 7.1 AWS AppConfig Layered Configuration

AWS AppConfig implements a sophisticated layering model:

```yaml
# AppConfig hierarchy
Application → Environment → Configuration Profile → Deployments

# Merge strategy:
# 1. Start with application defaults
# 2. Override with environment-specific values
# 3. Apply profile-specific overrides
# 4. Use deployment-specific values (highest priority)
```

### 7.2 HashiCorp Consul Template Merging

Consul template uses JSON Merge Patch for template rendering:

```hcl
# consul-template.hcl
template {
  source = "config.json.tmpl"
  destination = "/etc/app/config.json"
  command = "systemctl restart app"

  # Use RFC 7396 merge patch
  left = "/etc/app/defaults.json"
  right = "{{ key \"app/config\" }}"
}
```

## 8. Algorithm Selection Guide

| Use Case | Recommended Algorithm | Pros | Cons |
|----------|----------------------|------|------|
| Simple key-value overrides | Precedence-based | Simple, fast | Limited merging capability |
| Nested configuration | Post-order tree traversal | Deep merge support | More complex implementation |
| Schema-aware merging | Strategic merge patch | API-aware conflicts | Requires schema knowledge |
| Configuration as code | JSON Merge Patch | Standard, toolable | No array merge support |

## 9. Testing Strategies

### 9.1 Merge Testing Patterns

```typescript
describe('ConfigurationMerger', () => {
  const merger = new LayeredMerger();

  it('should merge layers with correct precedence', () => {
    merger.addLayer({
      id: 'defaults',
      priority: 1,
      config: { port: 3000 }
    });

    merger.addLayer({
      id: 'env',
      priority: 2,
      config: { port: 8080 }
    });

    const result = merger.merge();
    expect(result.port).toBe(8080); // Higher priority wins
  });

  it('should handle null values as deletion', () => {
    merger.addLayer({
      id: 'remove-secret',
      priority: 1,
      config: { secret: null }
    });

    const result = merger.merge({ secret: 'value' });
    expect(result.secret).toBeUndefined();
  });
});
```

### 9.2 Integration Testing

```typescript
it('should integrate with file system layers', async () => {
  const fs = require('fs').promises;

  // Simulate file-based configuration
  const layers = [
    {
      id: 'base',
      priority: 1,
      config: await fs.readFile('config/base.json', 'utf8')
    },
    {
      id: 'env',
      priority: 2,
      config: await fs.readFile('config/dev.json', 'utf8')
    }
  ];

  const result = merger.merge(layers);
  // Verify merged configuration
});
```

## Conclusion

Multi-layer configuration merge orchestration requires careful consideration of:

1. **Precedence models**: Choose appropriate layering strategy
2. **Merge algorithms**: Select right algorithm for data structure
3. **Error handling**: Plan for conflicts and validation
4. **Performance**: Consider caching and optimization
5. **Testing**: Ensure deterministic results

RFC 7396 provides a good standard for simple JSON merging, while tree-based approaches work better for nested configurations. Always validate results and document precedence rules clearly.

## References (To be populated when web search is available)

- RFC 7396: JavaScript Object Notation (JSON) Merge Patch
- Docker Compose file documentation
- Kubernetes ConfigMap documentation
- Spring Boot configuration precedence
- Git merge algorithm documentation