# Memory System

Persistent storage for facts, entities, and relationships.

## Architecture

The memory system combines three storage backends:

```
┌──────────────────────────────────────────────────────────────┐
│ MemoryStore                                                  │
│                                                               │
│  conn: Mutex<Connection>   ── SQLite (memory.db)              │
│  graph: Option<GraphStore> ── graphqlite (graph.db)           │
│  vectors_initialized: bool ── sqlite-vec (vec0 table)         │
└──────────────────────────────────────────────────────────────┘
```

### Storage Components

| Component | Purpose | Technology |
|-----------|---------|------------|
| **SQLite** | Fact storage, sessions, notes | rusqlite |
| **sqlite-vec** | Vector similarity search | vec0 virtual table |
| **graphqlite** | Entity relationships | Cypher-like queries |

## Memory Types

### Facts

Individual pieces of information with metadata:

```rust
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub source: MemorySource,
    pub metadata: MemoryMetadata,
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub reinforcement_count: u32,
    pub superseded: bool,
    pub superseded_by: Option<String>,
}
```

### Memory Sources

| Source | Base Confidence | Description |
|--------|-----------------|-------------|
| `Stated` | 1.0 | User explicitly stated |
| `System` | 0.9 | System-derived |
| `Observed` | 0.7 | Inferred from behavior |
| `Inferred` | 0.5 | Logical deduction |

### Entities

Named entities stored in the graph:

```rust
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,  // person, project, concept, etc.
    pub properties: HashMap<String, String>,
}
```

### Relationships

Connections between entities:

```rust
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub relation_type: String,  // knows, works_on, depends_on, etc.
    pub properties: HashMap<String, String>,
}
```

## Confidence Scoring

Dynamic scoring based on multiple factors:

```
score = base × reinforcement × staleness

base: stated=1.0, system=0.9, observed=0.7, inferred=0.5
reinforcement: min(1 + 0.1n, 1.5)  where n = reinforcement_count
staleness: linear decay to 0.3 over 365 days (fresh < 30 days)
superseded: always 0.0
```

### Scoring Examples

| Fact | Age | Reinforced | Score |
|------|-----|------------|-------|
| Stated fact, 5 days old | Fresh | 0 | 1.0 |
| Stated fact, 5 days old | Fresh | 3 | 1.3 |
| Observed fact, 200 days old | Stale | 0 | 0.45 |
| Superseded fact | Any | Any | 0.0 |

## Recall (Hybrid Search)

Recall combines multiple signals:

```
final_score = similarity * 0.4 + graph_score * 0.3 + confidence * 0.3
```

### Recall Process

1. **Embed query** — Generate vector for search query
2. **Vector search** — Find similar embeddings via sqlite-vec
3. **Graph expansion** — Find related entities
4. **Score combination** — Weighted merge of signals
5. **Threshold filter** — Remove low-scoring results
6. **Rank and limit** — Return top N results

### Recall Configuration

```rust
pub struct RecallConfig {
    pub limit: usize,           // Max results (default: 5)
    pub threshold: f32,         // Min score (default: 0.6)
    pub include_graph: bool,    // Use graph expansion
}
```

## Contradiction Detection

When storing facts, Arawn detects contradictions:

```
1. Find facts with same subject + predicate
2. If identical content → Reinforce (bump count)
3. If different content → Supersede old fact
4. If no match → Insert new fact
```

### Example

```
Existing: "David's language is Python"
New:      "David's language is Rust"
Result:   Old fact superseded, new fact inserted
```

## Graph Operations

### Adding Entities

```rust
graph.add_entity(Entity {
    name: "Arawn".to_string(),
    entity_type: "project".to_string(),
    properties: HashMap::from([
        ("language", "Rust"),
        ("type", "agent"),
    ]),
})?;
```

### Adding Relationships

```rust
graph.add_relationship(Relationship {
    source: "david".to_string(),
    target: "arawn".to_string(),
    relation_type: "created".to_string(),
    properties: HashMap::new(),
})?;
```

### Querying Neighbors

```rust
let neighbors = graph.get_neighbors("arawn", Some("depends_on"))?;
// Returns: [tokio, rusqlite, axum, ...]
```

## Best Practices

See [Memory Guidelines](../reference/memory-guidelines.md) for:
- When to store vs. not store
- Memory hygiene patterns
- Tool-specific guidance
