# Stability

Drachenhauch gets a new version often, usually several times a month
(`2026.51`, `2026.52` …). So that a program that runs today also runs with
the later ones, these promises hold since 2026.52. What changed for existing
programs in each version is listed in the [change log](aenderungen.md).

## What is fixed

| What | Promise |
|---|---|
| Language | Keywords, types, operators and their meaning stay. A program that compiles today also compiles with the next version -- except for the cases under [If something does break](#if-something-does-break). |
| Command names | No command disappears. If one gets a new name following the [guideline](builtins-namen.md), the old one keeps working, with no expiry date. |
| Arguments | What a command takes today, it keeps taking. New parameters are added at the end and are optional. |
| Parameter names | The names for named arguments (`CIRCLE(x: 10, y: 20, r: 5)`) stay as they are. |
| Files | What a program writes (save games with `save`, `.dhform` forms, Studio projects) is read by every later version as well. |
| Exported programs | They carry their runtime with them and keep running, whatever comes out afterwards. |
| Command line | `dhrt run`, `dhrt --export`, `dhrt --check`, `dhrt test` and their switches stay. A default only changes when the result does the same (since 2026.52 `dhrt --export` uses the smallest runtime that fits; `--schlank` still works, `--voll` takes the full one). |

## What may change

| What | Why |
|---|---|
| Wording and language of messages | Messages get more precise, and dhrt speaks the language of the system. What a program sees in a `CATCH` variable is always German regardless; still, a program does not check for the exact text. |
| New warnings | `dhrt --check` and the editor find more with every version. A warning does not stop a program. |
| Appearance | Themes, spacing and colours of the gui, the IDE and the Studio. |
| Precompiled bytecode (`.dhc`) | It belongs to its version and carries it (since 2026.52); another version rejects it with a message instead of running it differently without a word. Compile again after an update. |
| Tool settings | `ide.json`, `studio.json` and their relatives. |

## If something does break

A new version reports an error that used to compute something wrong silently
-- a clear stop is better than a wrong result. Every version names such
changes at the top of its release notes, under "What changes for existing
programs", collected in the [change log](aenderungen.md).

## Version numbers

Year and a running number (`2026.52`). There are no major and minor versions:
every version keeps the promises above, so no number is needed to announce a
break.
