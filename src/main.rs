//! gray-loop — self-driving iteration loops on plain wire primitives.
//!
//! Behavior-level reimplementation of mitsupi's `loop` extension
//! (proprietary — zero code copied): `/loop <goal>` seeds
//! `~/.gray/loop/<session>/state.json` and returns a `{prompt}` that puts
//! the agent to work. On every `agent/before_start` while the state reads
//! `running` and `iter < max`, the hook bumps `iter` and appends a
//! "continue" user message — so the loop terminates at `max` even if the
//! model never touches the file. The `loop_done` tool lets the agent stop
//! the loop early with `done`/`failed`. `/loop stop` forces `failed`,
//! `/loop max N` retunes the cap, `/loop status` reports.
//!
//! Fail open everywhere: malformed state, missing files, bad args — all
//! produce `{}` or a text note, never a crash.

use std::io::{BufRead, Write};
use std::path::PathBuf;

use serde_json::{Value, json};

const DEFAULT_MAX: u64 = 10;

fn manifest() -> Value {
    json!({
        "name": "loop",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "2.0",
        "tools": [{
            "name": "loop_done",
            "description": "Stop the active /loop when the goal is verifiably complete (status \"done\") or unachievable (status \"failed\"). Only call when a loop is running.",
            "parameters": {
                "type": "object",
                "properties": {
                    "status": { "type": "string", "enum": ["done", "failed"], "description": "Final loop status." },
                    "reason": { "type": "string", "description": "Why the loop is ending." }
                },
                "required": ["status"]
            }
        }],
        "commands": ["/loop"],
        "hooks": ["agent/before_start"],
    })
}

// ── state ─────────────────────────────────────────────────────

fn gray_home() -> PathBuf {
    std::env::var_os("GRAY_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".gray")))
        .unwrap_or_else(|| PathBuf::from(".gray"))
}

fn state_path(session_id: &str) -> PathBuf {
    // session ids are uuids, but sanitize anyway — never trust a filename
    let safe: String = session_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    gray_home().join("loop").join(safe).join("state.json")
}

fn read_state(session_id: &str) -> Option<Value> {
    let text = std::fs::read_to_string(state_path(session_id)).ok()?;
    serde_json::from_str(&text).ok()
}

fn write_state(session_id: &str, state: &Value) -> std::io::Result<()> {
    let path = state_path(session_id);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(state)?)?;
    std::fs::rename(&tmp, &path).or_else(|_| std::fs::write(&path, serde_json::to_string_pretty(state)?))
}

fn set_status(session_id: &str, status: &str, note: &str) -> bool {
    let Some(mut st) = read_state(session_id) else { return false };
    st["status"] = json!(status);
    if !note.is_empty() {
        st["notes"] = json!(note);
    }
    write_state(session_id, &st).is_ok()
}

fn session_id(params: &Value) -> String {
    params
        .get("session")
        .and_then(|s| s.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

// ── commands ──────────────────────────────────────────────────

/// `/loop <goal>` — seed state, return the kickoff prompt.
fn start_goal(session_id: &str, goal: &str) -> Value {
    let path = state_path(session_id);
    let existing_max = read_state(session_id)
        .and_then(|st| st.get("max").and_then(Value::as_u64))
        .unwrap_or(DEFAULT_MAX);
    let st = json!({
        "goal": goal,
        "iter": 0,
        "max": existing_max,
        "status": "running",
        "notes": "",
    });
    if let Err(e) = write_state(session_id, &st) {
        return json!({ "text": format!("loop: couldn't write {}: {e}", path.display()) });
    }
    json!({ "prompt": format!(
        "Work on this goal: {goal}\n\n\
         You are inside a /loop — after this turn you will be re-prompted to continue until the goal is\n\
         done or the iteration cap ({existing_max}) is hit. Keep a durable progress journal at\n\
         {} — a JSON object with fields \"iter\" (managed by the loop), \"status\"\n\
         (\"running\" | \"done\" | \"failed\"), and \"notes\" (a short progress summary for your next\n\
         iteration: what's done, what's next, anything you must not redo). Update \"status\" and\n\
         \"notes\" before ending every turn. When the goal is verifiably complete, call the loop_done\n\
         tool with status \"done\"; if it turns out to be unachievable, call it with \"failed\" and a reason.",
        path.display()
    ) })
}

fn run_command(argv: &[&str], params: &Value) -> Value {
    let sid = session_id(params);
    if sid.is_empty() {
        return json!({ "text": "loop: no session id" });
    }
    match argv.first().copied() {
        Some("stop") => {
            if set_status(&sid, "failed", "stopped by user via /loop stop") {
                json!({ "text": "loop stopped (status → failed)" })
            } else {
                json!({ "text": "no active loop for this session" })
            }
        }
        Some("status") => match read_state(&sid) {
            Some(st) => json!({ "text": format!(
                "loop {} — iter {}/{} — goal: {}\nnotes: {}",
                st.get("status").and_then(Value::as_str).unwrap_or("?"),
                st.get("iter").and_then(Value::as_u64).unwrap_or(0),
                st.get("max").and_then(Value::as_u64).unwrap_or(DEFAULT_MAX),
                st.get("goal").and_then(Value::as_str).unwrap_or("?"),
                st.get("notes").and_then(Value::as_str).unwrap_or(""),
            ) }),
            None => json!({ "text": "no loop state for this session" }),
        },
        Some("max") => {
            let Some(n) = argv.get(1).and_then(|s| s.parse::<u64>().ok()).filter(|n| *n > 0) else {
                return json!({ "text": "usage: /loop max <positive N>" });
            };
            match read_state(&sid) {
                Some(mut st) => {
                    st["max"] = json!(n);
                    if write_state(&sid, &st).is_ok() {
                        json!({ "text": format!("loop max → {n}") })
                    } else {
                        json!({ "text": "loop: couldn't write state" })
                    }
                }
                None => json!({ "text": "no active loop — start one with /loop <goal> first" }),
            }
        }
        Some(goal_word) => {
            let goal = std::iter::once(goal_word)
                .chain(argv.iter().skip(1).copied())
                .collect::<Vec<_>>()
                .join(" ");
            start_goal(&sid, &goal)
        }
        None => json!({ "text": format!(
            "gray-loop {} — /loop <goal> runs the goal in self-driving iterations. \
             /loop status · /loop stop · /loop max N",
            env!("CARGO_PKG_VERSION")
        ) }),
    }
}

// ── hook + tool ───────────────────────────────────────────────

/// `agent/before_start` — while a loop is running under its cap, bump iter
/// and append the continue message. Anything odd → `{}` (fail open).
fn before_start(params: &Value) -> Value {
    let sid = session_id(params);
    if sid.is_empty() {
        return json!({});
    }
    let Some(mut st) = read_state(&sid) else { return json!({}) };
    if st.get("status").and_then(Value::as_str) != Some("running") {
        return json!({});
    }
    let iter = st.get("iter").and_then(Value::as_u64).unwrap_or(0);
    let max = st.get("max").and_then(Value::as_u64).unwrap_or(DEFAULT_MAX);
    if iter >= max {
        st["status"] = json!("failed");
        st["notes"] = json!(format!("iteration cap ({max}) reached"));
        let _ = write_state(&sid, &st);
        return json!({});
    }
    st["iter"] = json!(iter + 1);
    if write_state(&sid, &st).is_err() {
        return json!({});
    }
    let goal = st.get("goal").and_then(Value::as_str).unwrap_or("?");
    let notes = st.get("notes").and_then(Value::as_str).unwrap_or("");
    let notes = if notes.is_empty() { String::new() } else { format!(" Progress so far: {notes}") };
    json!({ "text": format!(
        "Loop iteration {} of \"{goal}\": continue.{notes} \
         Call the loop_done tool when the goal is verifiably complete.",
        iter + 1
    ) })
}

fn call_tool(name: &str, args: &Value, params: &Value) -> Result<String, String> {
    match name {
        "loop_done" => {
            let sid = session_id(params);
            if sid.is_empty() {
                return Err("no session id".into());
            }
            let status = args.get("status").and_then(Value::as_str).unwrap_or("");
            if !matches!(status, "done" | "failed") {
                return Err("status must be \"done\" or \"failed\"".into());
            }
            let reason = args.get("reason").and_then(Value::as_str).unwrap_or("");
            match read_state(&sid) {
                Some(st) if st.get("status").and_then(Value::as_str) == Some("running") => {
                    if set_status(&sid, status, reason) {
                        Ok(format!("loop ended: {status}{}", if reason.is_empty() { String::new() } else { format!(" — {reason}") }))
                    } else {
                        Err("couldn't write loop state".into())
                    }
                }
                Some(_) => Ok("loop is not running".into()),
                None => Ok("no active loop is running".into()),
            }
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

// ── wire ──────────────────────────────────────────────────────

fn handle(req: &Value) -> (Option<Value>, bool) {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let Some(id) = id else {
        return (None, method == "plugin/shutdown");
    };
    let result = match method {
        "plugin/manifest" => manifest(),
        "agent/before_start" => before_start(&params),
        "command/run" => {
            let argv: Vec<&str> = params
                .get("argv")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            run_command(&argv, &params)
        }
        "tool/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("args").cloned().unwrap_or(Value::Null);
            match call_tool(name, &args, &params) {
                Ok(text) => json!({ "content": text }),
                Err(text) => json!({ "content": text, "is_error": true }),
            }
        }
        "plugin/shutdown" => return (Some(json!({ "id": id, "result": {} })), true),
        _ => {
            let error = json!({ "code": -32601, "message": "method not found" });
            return (Some(json!({ "id": id, "error": error })), false);
        }
    };
    (Some(json!({ "id": id, "result": result })), false)
}

fn main() -> std::io::Result<()> {
    if std::env::args().nth(1).as_deref() == Some("manifest") {
        println!("{}", manifest());
        return Ok(());
    }
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        let Ok(req) = serde_json::from_str::<Value>(&line) else { continue };
        let (reply, exit) = handle(&req);
        if let Some(reply) = reply {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
        if exit {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(method: &str, params: Value) -> Value {
        handle(&json!({ "id": 1, "method": method, "params": params })).0.unwrap()
    }

    /// Point state_path at a scratch GRAY_HOME for the duration of `f`.
    /// Env vars are process-global; tests touching them must serialize.
    fn with_home<T>(tag: &str, f: impl FnOnce(&std::path::Path) -> T) -> T {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _g = LOCK.lock().unwrap();
        let home = std::env::temp_dir().join(format!("grayloop-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        unsafe { std::env::set_var("GRAY_HOME", &home) };
        let out = f(&home);
        unsafe { std::env::remove_var("GRAY_HOME") };
        let _ = std::fs::remove_dir_all(&home);
        out
    }

    #[test]
    fn manifest_claims_hook_tool_command() {
        let m = call("plugin/manifest", Value::Null)["result"].clone();
        assert_eq!(m["name"], "loop");
        assert_eq!(m["protocol"], "2.0");
        assert_eq!(m["hooks"], json!(["agent/before_start"]));
        assert_eq!(m["tools"][0]["name"], "loop_done");
    }

    #[test]
    fn loop_lifecycle_start_nudge_done() {
        with_home("life", |home| {
            let sess = json!({ "id": "s-1", "cwd": "/x" });
            // /loop <goal> → prompt + state file
            let r = call("command/run", json!({ "name": "/loop", "argv": ["fix", "tests"], "session": sess }));
            assert!(r["result"]["prompt"].as_str().unwrap().contains("fix tests"));
            let st: Value = serde_json::from_str(
                &std::fs::read_to_string(home.join("loop/s-1/state.json")).unwrap(),
            ).unwrap();
            assert_eq!(st["status"], "running");
            // before_start nudges and counts
            let r = call("agent/before_start", json!({ "session": sess }));
            assert!(r["result"]["text"].as_str().unwrap().contains("Loop iteration 1"));
            // loop_done stops it
            let r = call("tool/call", json!({
                "name": "loop_done",
                "args": { "status": "done", "reason": "all green" },
                "session": sess,
            }));
            assert!(r["result"]["content"].as_str().unwrap().contains("done"));
            // no more nudges
            let r = call("agent/before_start", json!({ "session": sess }));
            assert_eq!(r["result"], json!({}));
        });
    }

    #[test]
    fn iteration_cap_forces_failed() {
        with_home("cap", |_| {
            let sess = json!({ "id": "s-2" });
            call("command/run", json!({ "name": "/loop", "argv": ["g"], "session": sess }));
            call("command/run", json!({ "name": "/loop", "argv": ["max", "2"], "session": sess }));
            for expect_nudge in [true, true, false] {
                let r = call("agent/before_start", json!({ "session": sess }));
                assert_eq!(r["result"].get("text").is_some(), expect_nudge);
            }
            let st = read_state("s-2").unwrap();
            assert_eq!(st["status"], "failed");
        });
    }

    #[test]
    fn stop_and_status_and_fail_open() {
        with_home("stop", |_| {
            let sess = json!({ "id": "s-3" });
            // no loop: everything degrades to text/{}
            let r = call("agent/before_start", json!({ "session": sess }));
            assert_eq!(r["result"], json!({}));
            let r = call("command/run", json!({ "name": "/loop", "argv": ["status"], "session": sess }));
            assert!(r["result"]["text"].as_str().unwrap().contains("no loop"));
            // start then stop
            call("command/run", json!({ "name": "/loop", "argv": ["x"], "session": sess }));
            let r = call("command/run", json!({ "name": "/loop", "argv": ["stop"], "session": sess }));
            assert!(r["result"]["text"].as_str().unwrap().contains("stopped"));
            let st = read_state("s-3").unwrap();
            assert_eq!(st["status"], "failed");
        });
    }

    #[test]
    fn loop_done_rejects_bad_status() {
        let r = call("tool/call", json!({
            "name": "loop_done", "args": { "status": "maybe" }, "session": { "id": "s-9" }
        }));
        assert_eq!(r["result"]["is_error"], true);
    }

    #[test]
    fn shutdown_replies_then_exits() {
        let (reply, exit) = handle(&json!({ "id": 2, "method": "plugin/shutdown" }));
        assert!(reply.is_some() && exit);
    }
}
