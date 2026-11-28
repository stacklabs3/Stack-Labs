---
name: maintain-formal-verification-skill
description: "Bring a TLA+ verification in verificacao/<fluxo>/ back in line with the code after the code changes: find the commits that touched covered code, update the model or invariants to describe the code as it is now, rerun, and say whether each failure is drift in the model or a bug in the code. Use for /maintain-formal-verification-skill, \"is the model still true to the code\", or after merges and refactors that touched covered code without rechecking it."
disable-model-invocation: true
---

# Maintain a formal verification

<!-- Vertente da StackLabs: cópia de cursor/plugins @ b1fbf4b (pstack, MIT), com os remendos de REMENDOS.md. -->

A formal check proves an invariant of what it describes. When the code changes and the model does not, the check can keep passing while it describes code that no longer exists. That pass is the dangerous case, because it looks like a proof. This skill is the upkeep loop for a folder `verificacao/<fluxo>/` that `/create-formal-verification-skill` left: the `.tla`, its `.cfg` files, `check.sh` and the `README.md` with the coverage record. It handles the changes that skipped the check: merges, edits made without it, and drift that builds up across many commits.

## Outcomes

Give each check one outcome, and say which:

- **in sync**: no change since the last sync alters what the check describes, and the check passes.
- **updated**: the code changed, the model now describes the new code, and the check passes.
- **bug**: the model describes the current code, and the code breaks the invariant. This is a finding about the code. It is not drift.
- **blocked**: the check could not be brought back in line or could not run. Say exactly why.

A check can be both updated and bug: the update makes the model describe the code again, and then the invariant fails on it.

## Edit scope

Edit only the verification: the `.tla`, the `.cfg` files, `check.sh` and the `README.md` of the folder. Never edit product code during a run, unless the user asks you to. When an invariant fails on the current code, do not weaken the invariant or the model to make it pass. Report the failure.

## Pass

1. **Find the checks.** They live in `verificacao/*/`. Read the `README.md` of each one, or of the one the user named. For each check, collect the invariants and the `RK<n>` each guards, the code it covers, the bounds and assumptions, the command that runs it, and the sync commit. If no folder exists, stop and point at `/create-formal-verification-skill`. If the README does not record a sync commit, use the last commit that changed the folder, and say that you did.

2. **Run it as it is.** Run `check.sh` once before you change anything, and record the result. A pass here does not tell you that the code is correct. It tells you that the old description still holds. A failure here can be drift or a bug. Do not decide which until step 5.

3. **Read every change to covered code.** Diff the covered files from the sync commit to the current commit, and read each commit message. Also look for code that moved: a covered function that is renamed, split, or moved to another file is still covered code. Also look for new code that touches the state that a check guards, such as a new function that writes the same fields. For each change, decide which of these it is:
   - **No effect.** The change does not alter anything the model describes, such as formatting, comments, or code paths that the model does not cover. Check this against the model, not against the commit message.
   - **The model no longer describes the code.** A step that was one transaction is now two, a branch was added or removed, a check moved, a new path reaches the guarded state, or a value now has a different range. The model must change.
   - **The promise changed on purpose.** The change alters what the code promises, and evidence in the change says that this is intended: the doc comment, the tests, the changelog, the commit message, or the `RK<n>` row. The invariant must change, and you must cite that evidence.
   - **The promise may be broken.** The code changed, and no evidence says that the promise changed. Treat a failure that follows as a bug candidate.

4. **Update from the code, not from the result.** Change the model so that each changed action again points to code that a reviewer can open, and keep one transaction or one lock in the code as one action in the model. Write the update from the code before you rerun, and do not shape it by what makes the check pass. Change an invariant only for a promise that changed on purpose. Write down the old wording, the new wording, and the evidence. Keep the bounds unless the change makes them wrong. If you must reduce a bound, say so in the report.

5. **Rerun and judge each result.**
   - **Pass:** the check can now pass on a description that has lost the ability to fail. Run every `quebra-*.cfg` and see each fail on the invariant it guards; for a new guard, add its `quebra-<guarda>.cfg` and mirror it in the code. `check.sh` must also show every action of `Next` with steps and no TLC warning. Then drop the breaks in the code.
   - **Fail:** replay the counterexample against the real code, step by step, with the project's fake of the outside service. If a step cannot happen in the code, the model is wrong. Fix the model, and say what was wrong. If every step can happen, it is a bug. Name the commit that introduced it, tell the counterexample as a sequence of steps in the code's own words, and state the harm in one sentence. Write the replay as a test that cites the `RK<n>` and fails, open the bug issue with `needs-triage`, and ask the human to run `/triage`.

6. **Have the table checked.** A subagent that does not see your reasoning receives the `.tla`, the action→code table and the code, and checks every row: the code exists, the names match, and each indivisible step is justified by a transaction or a lock.

7. **Update the README.** Update the code that each action covers, the commands, the bounds, the assumptions, and the sync commit. Set the sync commit to the commit that you checked. Do not move the sync commit for a blocked check, and say so in the report. Then run `check.sh` once more as the next agent would.

## Report

Tell the user, for each check:

- the outcome, and the commits that you read;
- what changed in the model and why, with a short reason for each change: the code changed shape, or the promise changed on purpose (with the evidence);
- the rerun result, and the breaks that show the updated check can still fail;
- each bug, in plain words: the counterexample as steps in the code's words, the harm, the `RK<n>`, and the commit that introduced it;
- what the model still does not cover, including new code that it should cover and does not.

Say outright which failures were drift in the model and which were bugs in the code. For a check that passed before you updated it and fails after, say that the old pass was on a stale description.

Put the verification updates in a change of their own, separate from any product fix.
