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

## Tags

`gray` `plugin` `loop` `rust`

---

Part of the [gray](https://github.com/vstaln/gray) plugin ecosystem —
the open-source AI agent harness. <https://gray.alignment.id>
