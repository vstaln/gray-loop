<div align="center">
  <img alt="gray-loop" src="assets/icon.svg" width="120" height="120" />
  <h1>gray-loop</h1>
  <p><strong>/loop [interval] &lt;prompt&gt; — scheduled and self-paced loops for gray.</strong></p>
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

`/loop [interval] <prompt>` — Claude Code's `/loop`, rebuilt for gray — plus a
checklist goal loop that runs until the work verifiably passes.

A sidecar plugin for [gray](https://github.com/vstaln/gray), scaffolded by
[gray-account](https://github.com/vstaln/gray-account).

## Usage

```
/loop 5m check the deploy            re-run every 5m (also: "... every 2 hours")
/loop babysit PR 1234 until it merges   self-paced: the agent picks each delay
/loop                                self-paced over .gray/loop.md (or ~/.gray/loop.md)
/loop goal <goal> [--gate "cmd"] [--review]   checklist loop until it verifiably passes
/loop status · stop · steer <msg> · max N · tasks · list
```

Interval parsing follows Claude Code: a leading `^\d+[smhd]$` token, else a
trailing `every <N><unit>` / `every <N> <unit-word>` clause (`check every PR`
has no interval), else no interval → self-paced. `--max N` caps ticks
(default 50). Fixed intervals clamp to 10s–7d.

## How it works

The first tick is the command's `{prompt}`. When a tick's turn ends
(`event/notify turn_end`, dispatched even in lean mode), the sidecar arms
a timer; when it fires, the next tick is queued in
`<gray home>/cron/inbox/<session>/loop-*.json` and the REPL's inbox poller
wakes the session with it — while the session stays open, like Claude Code.

- **Fixed** (`/loop 5m …`) — the next tick arms itself every interval.
- **Self-paced** (`/loop …`) — *pull*, not push, like Claude Code's
  `ScheduleWakeup`: each tick the model must re-arm with
  `gray loop wakeup <secs> [--reason …] [--noop]` (0 = continue now, up to
  3600) or end it with `gray loop stop "<reason>"`. No re-arm → the loop
  ends after that tick. The `loop_wakeup` tool does the same.
- Control goes through **bash** because some providers (claude-sub's MCP
  bridge) only forward an allowlisted set of host tools — bash is the one
  every model has. `$GRAY_SESSION_ID` picks the session.
- `host/say` prints "tick N in 5m" / "loop ended" lines (`host.say`).
- `/loop stop` bumps nothing — it flips `status`, and stale timers notice
  the state change (each arming carries an epoch) and exit.
- `/loop steer <msg>` rides the next tick's prompt.

## Goal loop (`/loop goal …`)

The checklist loop from 0.2 (now behind the `goal` word, or implied by
`--gate`/`--review`): state in `~/.gray/loop/<session>/state.json`, tasks as
the deterministic exit oracle (`loop_task` / `gray loop task add|pass|fail`),
learnings (`loop_learn` / `gray loop learn`), `loop_done` /
`gray loop done|fail`, a background `--gate` command that must exit 0, and an
optional `--review` second-agent stage (`VERDICT: CLEAR|RESOLVABLE|NOVEL`,
NOVEL escalates to a `host/ask` modal). Iteration cap default 10.

State honors `$GRAY_HOME` first, then `$HOME/.gray`. Shutdown only reaps the
sessions this sidecar served. Everything fails open.

## Wire

`command/run` (`/loop`) + `event/notify` (`turn_end`) + `agent/before_start`
(hook) + `tool/call` (`loop_wakeup`, `loop_done`, `loop_task`, `loop_learn`) +
the `gray loop <verb>` CLI. Protocol `2.0` (mutating hook). Declares
`capabilities: ["host.ask", "host.say"]` — the ask for the human-review step; the ask
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
