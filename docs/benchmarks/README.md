# Benchmark Protocol & Measured Results

The claim **"fewer tokens and fewer architectural violations with clrinf"** is measured in two layers: deterministic context/guardrail measurements and end-to-end LLM A/B runs.

## 1. Deterministic Context Cost (`clrinf bench context`)

```bash
clrinf bench context <event> --schema-dir <schemas> --src <source-roots...>
```

What it measures:
- **Baseline:** All source files under `--src` plus all schemas (simulating full-repo agent exploration).
- **Slice:** Target event schema + hand-written files referencing the event + generated agent rules block.
- **MCP schema cost:** Fixed per-session token overhead of `full`, `events`, and `lean` tool definitions.

Token counts use the `~4 chars/token` estimate; ratios are what matter rather than absolute counts.

> [!WARNING]
> `bench context` measures **read-side** context only. Write-side tokens and retry turns appear only in an end-to-end A/B run (Section 3). If no hand-written file references the event yet, the slice consists only of the schema and looks artificially small; do not cite empty-slice ratios.

## 2. Measured Deterministic Results (`scripts/bench-guardrails.ps1` / `.sh`)

The values below are directly measured and reproducible in this repository via `.\scripts\bench-guardrails.ps1` (Windows) or `bash scripts/bench-guardrails.sh` (Linux/macOS):

### A. Polyglot Guardrails & Event Context Slicing (`examples/guardrails-benchmark/`)
- **Clean baseline (`clrinf verify`):** `0` errors across Rust (`orders`), TypeScript (`billing`), and C# (`inventory`).
- **Context cost (`orders.placed.v1`):**
  - Baseline (`services/` + schemas): **2,015 tokens** (15 files)
  - `clrinf` slice (schema + 3 hand-written affected files + rules block): **1,246 tokens** (4 files) → **38.2% reduction**
- **Fixed per-session MCP tool-schema overhead:**
  - `full` (27 tools): **3,698 tokens**
  - `events` (14 tools): **1,983 tokens**
  - `lean` (10 tools): **1,440 tokens** (**61.1% lower fixed overhead** vs. `full`)
- **Seeded AI-agent failure detection (5/5 caught deterministically):**
  1. Forbidden domain `Infrastructure` import (`RULE_FORBIDDEN_DEPENDENCY`, `scope = "imports"`) + Claude Code `PostToolUse` hook returning **exit code `2`**.
  2. Direct outbox bypass (`DRIFT_RAW_OUTBOX_WRITE`).
  3. Breaking `v2` schema evolution without an upcaster (`TOPOLOGY_UPCASTER_MISSING`).
  4. Cross-module internal import bypassing public contracts (`RULE_MODULE_BOUNDARY_VIOLATION`).
  5. Hand-editing a generated file (`RULE_GENERATED_FILE_MODIFIED` via Claude Code `PostToolUse` hook **exit code `2`**).

### B. Compile-Time Native AOT CRUD vs. Disk-Generated CRUD (`Category` + `Brand`)
- **Disk-generated CRUD (`clrinf add entity --pattern vertical-slice`):** **3,850 tokens** across `16` `.cs` files.
- **Native AOT CRUD (`core/csharp/examples/AotCrud/Domain/*.cs`):** **356 tokens** across `3` `.cs` files → **90.8% reduction** in on-disk agent context.

---

## 3. End-to-End Agent A/B Protocol

Run the same task, same model, and same starting commit with `clrinf` disabled vs. enabled:

| Arm | Setup |
|---|---|
| A (control) | No `clrinf`; agent explores the repository manually |
| B (`clrinf`) | `clrinf init --guardrails` (`agents sync`, `hook install`, MCP `lean` profile) |

### Per-Tool Arm B Setup

Volunteers may run different coding agents as long as Arm A and Arm B use the same tool and model:

| Agent | Rules file | Hook install | MCP |
|---|---|---|---|
| Claude Code | `CLAUDE.md` | `clrinf hook install --agent claude` | `clrinf mcp --profile lean` |
| Cursor | `.cursor/rules/clrinf.mdc` | `clrinf hook install --agent cursor` | `clrinf mcp --profile lean` in `.cursor/mcp.json` |
| Antigravity | `GEMINI.md` / `AGENTS.md` | `clrinf hook install --agent antigravity` | `clrinf mcp --profile lean` |
| Other / hookless | `AGENTS.md` | `clrinf hook install --agent git` (pre-commit) | `clrinf mcp --profile lean` |

To confirm hooks are active before recording runs, inject a forbidden import into a domain file and verify the agent sees the violation diagnostic. (As noted in [Agent Guardrails](../guides/agent-guardrails.md), the Antigravity hook format has not yet been verified against live official docs; mark runs accordingly if inactive.)

Recommended tasks (at least 5 repeats each):
1. Subscribe a new service to an existing event.
2. Add a required field to an event schema (`v1` → `v2` with upcaster).
3. Implement a feature that tempts the agent to import infrastructure inside the domain layer.

Metrics to record (report median and range, plus exact model/version):
- Total tokens (input + output)
- Turn / tool-call count
- Architectural violations at completion (`clrinf verify --json` → `errorsCount`)
- Task pass rate against pre-written acceptance tests
- Manual human lines edited to fix violations
