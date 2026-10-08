# gray-loop

Self-driving iteration loops built on plain wire primitives: `/loop <goal>`
keeps re-prompting the agent until the goal is verifiably done, the cap is
hit, or it's stopped. Behavior-level reimplementation of mitsupi's `loop`
extension (proprietary — zero code copied).

A sidecar plugin for [gray](https://github.com/vstaln/gray), scaffolded by
[gray-account](https://github.com/vstaln/gray-account).

## What it does

- `/loop <goal>` — seeds `~/.gray/loop/<session>/state.json`
  (`{goal, iter, max, status, notes}`) and returns a `{prompt}` putting the
  agent to work on the goal with instructions to keep `status`/`notes`
  current at that path.
- `agent/before_start` hook — while the state reads `running` and
  `iter < max`, bumps `iter` and appends a "Loop iteration N of \<goal\>:
  continue … call loop_done when verifiably complete" user message.
  Counting lives in the hook, so a model that never writes state still
  terminates at the cap.
- `loop_done` tool — `{status: "done"|"failed", reason}` ends the loop.
- `/loop status` — report goal, iter/max, status, notes.
- `/loop stop` — force status `failed`.
- `/loop max N` — retune the iteration cap (default 10).

State honors `$GRAY_HOME` first, then `$HOME/.gray`. Everything fails open:
odd state files, missing sessions, bad args → `{}` or a text note, never a
crash.

## Wire

`command/run` (`/loop`) + `agent/before_start` (hook) + `tool/call`
(`loop_done`). Protocol `2.0` (mutating hook).

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
