<div align="center">
  <img alt="gray-loop" src="assets/icon.svg" width="120" height="120" />
  <h1>gray-loop</h1>
  <p><strong>Self-driving iteration loops: /loop <goal> keeps going until the goal is done.</strong></p>
  <p>
    <a href="https://gray.alignment.id">Website</a> ·
    <a href="https://gray.alignment.id/plugins/gray-loop">Store</a> ·
    <a href="https://github.com/vstaln/gray-loop">Source</a> ·
    <a href="https://github.com/vstaln/gray">gray</a>
  </p>
  <p>
    <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-1c1c20?style=flat-square&labelColor=0a0a0b" /></a>
    <a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-rust-1c1c20?style=flat-square&labelColor=0a0a0b&logo=rust&logoColor=d4a373" /></a>
    <a href="https://gray.alignment.id/plugins/gray-loop"><img alt="gray plugin" src="https://img.shields.io/badge/gray-plugin-1c1c20?style=flat-square&labelColor=0a0a0b&color=7aa2f7" /></a>
  </p>
</div>

<br/>

```bash
gray plugin install gray-loop
```

Self-driving iteration loops built on plain wire primitives: `/loop <goal>`
keeps re-prompting the agent until the goal is verifiably done, the cap is
hit, or it's stopped.

A sidecar plugin for [gray](https://github.com/vstaln/gray), scaffolded by
[gray-account](https://github.com/vstaln/gray-account).

## What it does

- `/loop <goal> [--review] [--gate "<verify cmd>"]` — seeds
  `~/.gray/loop/<session>/state.json` (`{goal, iter, max, status, notes,
  tasks[], learnings[], steer[], gate?, gate_run?, review?, review_run?,
  review_passed?, review_disabled?}`) and returns a `{prompt}` putting the
  agent to work on the goal.
- `agent/before_start` hook — while the state reads `running` and
  `iter < max`, bumps `iter` and appends a continue message carrying the
  goal, the task checklist, the latest learnings, any queued steering, and
  the gate reminder. Counting lives in the hook, so a model that never
  writes state still terminates at the cap.
- `loop_task` tool — the deterministic exit oracle: `add` a checkable unit,
  `pass`/`fail` it by 1-based id, exact text, unique prefix, or unique
  substring. The plugin ends the loop itself once every task passes; a
  manual `loop_done` is rejected while tasks are unchecked.
- `--gate "<cmd>"` — a verification command (runs `sh -c` from the session
  cwd, 180s cap). Once all tasks pass — or a taskless `loop_done` is called
  — the gate is armed in the background: combined output streams to
  `gate.out`, the exit code to `gate.code`. Polls (`loop_done`, the next
  `before_start` iteration) resolve it: exit 0 ends the loop, anything else
  rejects the "done" with the output tail and the loop keeps going. A
  running gate is killed on `/loop stop`, `loop_done "failed"`, cap-out, or
  timeout.
- `--review` — a second-agent stage ahead of the gate. Once tasks pass (or
  a taskless `loop_done` lands), a detached `gray -p` child reviews the
  real state of the work (binary: `$GRAY_LOOP_REVIEW_BIN`, else `$GRAY_BIN`,
  else `gray` on PATH; 600s cap; output to `review.out`, exit to
  `review.code`; the child shows up in the Agents view as
  `loop-review-*`). Its reply must end with one verdict line:
  `VERDICT: CLEAR` proceeds to the gate; `VERDICT: RESOLVABLE — <fix>`
  queues the finding as steering and rejects the "done"; `VERDICT: NOVEL —
  <issue>` escalates to a blocking `host/ask` modal — *send back to work*
  (optional guidance rides as a note), *ship anyway*, or *stop the loop*.
  No ask channel → the issue degrades to a steering note and the loop keeps
  going; a missing `gray` disables review outright. A failed gate clears
  `review_passed`, so post-gate fixes get re-reviewed.
- `loop_learn` tool — append a durable learning (last 3 ride every
  continue prompt; log capped at 50).
- `loop_done` tool — `{status: "done"|"failed", reason}` ends the loop;
  with a gate configured, "done" arms/polls it first.
- `/loop status` — report goal, iter/max, task tally, gate (incl. running),
  notes.
- `/loop tasks` — render the checklist.
- `/loop steer <msg>` — queue a steering message for the next iteration.
- `/loop stop` — force status `failed`.
- `/loop max N` — retune the iteration cap (default 10).
- `/loop list` — every session's loop, globbed from `~/.gray/loop/*/`.

State honors `$GRAY_HOME` first, then `$HOME/.gray`. Everything fails open:
odd state files, missing sessions, bad args → `{}` or a text note, never a
crash.

## Wire

`command/run` (`/loop`) + `agent/before_start` (hook) + `tool/call`
(`loop_done`, `loop_task`, `loop_learn`). Protocol `2.0` (mutating hook).
Declares `capabilities: ["host.ask"]` for the human-review step; the ask
payloads are built with the shared [`gray-questions`
crate](https://github.com/vstaln/gray-questions) (`git` dep, tag
`v0.1.1`).

## Install

```sh
gray plugin install loop
```

## Develop

```sh
cargo test
gray account check      # entry point + manifest handshake
gray account publish    # check → build → release → publish to the gray registry
```

Bump `version` in `Cargo.toml` before each `publish`; the registry refuses to
republish a version.

## Tags

`gray` `plugin` `loop` `rust`

---

Part of the [gray](https://github.com/vstaln/gray) plugin ecosystem —
the open-source AI agent harness. <https://gray.alignment.id>
