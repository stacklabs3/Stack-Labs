---
name: create-formal-verification-skill
description: "Model-check, with TLA+ and TLC, the design behind one RK risk of docs/riscos.md that tests rarely reach, and leave the model, its check and its coverage record in verificacao/<fluxo>/. Use for /create-formal-verification-skill, or when an approved RK of ordering, concurrency or repetition needs more than tests."
disable-model-invocation: true
---

# Create a formal verification

<!-- Vertente da StackLabs: cópia de cursor/plugins @ b1fbf4b (pstack, MIT), com os remendos de REMENDOS.md. -->

A test checks the cases that someone thought to write. Formal verification checks every case in a stated scope: every order of steps in a model of the design, up to the bounds of its constants. This skill takes one risk from the project's risk register, verifies the invariants that guard it, and leaves a folder that lets the next agent check a change against them. Write that folder for an agent who reads it cold, mid-change, and has never seen your work. The user must be able to trust the result and act on it without learning TLA+.

## Take the target from the risk register

The target is an `RK<n>` row of `docs/riscos.md` of the class time, ordering or concurrency, that the human approved for formal verification. If the risk does not exist yet, propose its row (class, damage in one sentence, origin, situation) and write it only after the human approves. Each invariant names the `RK<n>` it guards, in the comment above it and in the README.

Read the code of the flow from the risk, and ask the user only what you cannot observe.

## The tool is TLA+ with TLC

The tool is TLC from `tla2tools.jar` 1.7.4, pinned by sha256, run with `-workers 1` and `-coverage 1`. It needs only Java 8 or newer. The agent tells every counterexample in plain words itself, with no other tool.

## The folder and its names

Everything lives in `verificacao/<fluxo>/`, `<fluxo>` in the project's language, kebab-case:

- `<Modulo>.tla`: the model, with `Next` as a disjunction of named actions and one boolean constant per broken variant.
- one `.cfg` per variant: `<fluxo>.cfg` for the correct code, `quebra-<guarda>.cfg` for each guard broken on purpose, and one `.cfg` per bound of the ladder.
- `check.sh`: downloads the jar, checks the sha256, runs every `.cfg` and fails on the conditions of *Every action fires*.
- `README.md`: the coverage record (sync commit, files and functions covered, the action→code table with the justification of each indivisible step), the invariants and harms, what is left out, the bounds and how to run.

## What makes it good

- **True to the code.** Every action of the model points to code that a reviewer can open. What the model treats as one indivisible step is one transaction or one lock in the code, and the table says which. Two writes in the code are two actions in the model. Names come from this repo.
- **Able to fail, guard by guard.** A check that you have never seen fail proves nothing. For each guard the model claims to cover, add a broken variant (a removed condition, a reordered step) as a `quebra-<guarda>.cfg`, mirror it in the code, and see each invariant it guards fail. A guard that never makes an invariant fail leaves the coverage table.
- **Every action fires.** An action that never takes a step makes every invariant hold for free. `check.sh` fails when any action of `Next` has zero steps in the `-coverage` output of the correct `.cfg`, or when TLC prints a warning.
- **Checked by a second reader.** A subagent that does not see your reasoning receives the `.tla`, the table and the code, and checks every row: the code exists, the names match, and each indivisible step is justified by a transaction or a lock. Fix every row it rejects before you hand off.
- **Findings stay findings.** When the current code breaks an invariant, report it with the counterexample in the code's terms. Do not weaken the invariant or the model until it passes, and do not change product code unless the user asks.
- **Honest about scope.** Say what is verified, under which bounds and assumptions, and what is left out. A pass covers the model and its bounds, not the whole system.
- **Readable without the tool.** State each invariant and its harm in one plain sentence each. Tell a counterexample as a short sequence of steps in the code's words.
- **Cheap to rerun.** One command, a pinned tool version, and a smallest `.cfg` short enough that the next agent will run it on every relevant change. Record states, depth and wall time of every bound.
- **Kept in step with the code.** Record which files and functions each action covers, and the commit at which you last confirmed that the model describes them.

## Prove it before you hand off

Run `check.sh` on the correct `.cfg` and on every `quebra-*.cfg`. Each broken variant must fail on the invariant it guards, and the correct one must pass with every action fired and no warning. Then drop the scratch breaks in the code and confirm that the check is back where it started: passing, or failing only on the findings you will report.

## When the checker finds a counterexample

Replay it against the real code, step by step, with the project's fake of the outside service.

- **Every step can happen:** it is a bug. Write the replay as a test that cites the `RK<n>` and fails, open the bug issue with `needs-triage`, and ask the human to run `/triage`. The fix is another issue.
- **A step cannot happen:** the model is wrong. Fix the model from the code and record it as noise.

## Hand off

Tell the user what you verified, the bounds, what the check found in plain words, how to run it, and what it does not cover. The human reads the invariants, what is left out and the counterexample in words. Point the user at `/maintain-formal-verification-skill` for bringing the verification back in line after changes that did not recheck it.
