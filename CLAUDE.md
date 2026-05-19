# CLAUDE.md — Cuckoo

Behavioral guidelines for this Tauri + Rust + React/TypeScript project.
Derived from Andrej Karpathy's LLM coding guidelines, extended with project-specific constraints.

**Tradeoff:** These guidelines bias toward caution over speed. Use judgment on trivial tasks.

---

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop and name what's confusing.

For DB schema changes: always consider whether a migration guard is needed
(`pragma_table_info` check before `ALTER TABLE`). Never assume a column exists.

---

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No error handling for impossible scenarios (trust framework guarantees).
- If you write 200 lines and it could be 50, rewrite it.

Ask: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

---

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

- Don't refactor adjacent code unless it's broken.
- Match existing style: Rust closure pattern for transactions, `params![]` macros, etc.
- If you notice unrelated dead code, mention it — don't delete it.
- Remove imports/variables/functions that YOUR changes made unused.

Every changed line should trace directly to the request.

---

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

For every change, run the appropriate verifier before reporting done:
- Rust change → `cargo check --manifest-path src-tauri/Cargo.toml`
- TypeScript change → `npx tsc --noEmit`
- Both together when changing the Tauri IPC boundary (commands.rs + frontend types)

Multi-step plan format:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
```

---

## 5. Project-Specific Rules

### Rust / Database

**Transaction pattern** — `transaction()` requires `&mut Connection` which conflicts with
the `Mutex` guard. Always use the closure pattern instead:

```rust
conn.execute_batch("BEGIN")?;
let result: Result<()> = (|| {
    // all writes here
    Ok(())
})();
match result {
    Ok(_)  => { conn.execute_batch("COMMIT")?; Ok(()) }
    Err(e) => { conn.execute_batch("ROLLBACK").ok(); Err(e) }
}
```

Every function with more than one write MUST be wrapped in this pattern.

**Inventory writes** — `inventory_txns` INSERT and `inventory_batches` UPDATE must
always happen together inside the same transaction. Never write a txn without updating
the batch quantity, and vice versa.

**Number generation** — Use millisecond precision to avoid collisions:
`chrono::Local::now().format("%Y%m%d%H%M%S%3f")` for TXN/PO/PRD/STK numbers.

**wastage_rate convention** — Stored as a decimal (0.05 = 5%). Displayed and entered
as a percentage (0–100). Frontend must divide by 100 before sending, and multiply by 100
before displaying. Validate input is in range 0–100 before submitting.

**Soft-delete vs hard-delete:**
- Materials, suppliers, categories: soft-delete (`is_active = 0`). Always pre-check FK
  constraints (e.g. active materials in a category) and return an error if violated.
- Menu items: hard-delete with CASCADE (removes specs and station_menu_items).
- Never remove inventory_batches directly without also removing related inventory_txns.

**Query performance** — Never use per-row queries inside a loop (N+1). Use JOIN or a
single batch query + HashMap grouping in Rust.

### Printing

**Feie vs LAN** — Feie's HTTP API accepts plain UTF-8 text only. LAN printers accept
raw ESC/POS binary. Never call `from_utf8_lossy` on an `EscPosBuilder` buffer for Feie.
Use `build_*_text()` functions for Feie; use `builder.build()` bytes for LAN.

### TypeScript / React

**Data loading** — `loadData()` in `useAppData.ts` uses `Promise.allSettled`. One failed
fetch must never block the rest. Don't revert to `Promise.all` or sequential awaits.

**Post-mutation refresh** — Every `useAppActions.ts` handler that mutates data must call
`loadData()` after success. No silent state divergence between DB and UI.

**Types** — Frontend types in `src/types/index.ts` must stay in sync with Rust structs.
When adding a field to a Rust struct, add it to the matching TypeScript interface.

**Tauri IPC** — Command names use snake_case strings. Parameter names sent to Tauri use
camelCase JavaScript objects (Tauri's serde rename convention). Check both sides when
adding a new command.
