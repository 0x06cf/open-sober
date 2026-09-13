# Open Sober — Agent Instructions

## Branch strategy

- **`stable`** — Release channel. Only merged from `dev` after tests pass.
- **`dev`** — All development happens here. Single integration branch.
- **NO worktrees or feature branches.** Do not create worktrees or separate branches.
  Work on `dev` directly. Commit early, commit often.

## How to work

1. Always work on `dev` branch.
2. Never create git worktrees or feature branches.
3. If you need to research something, do it inline or in a temp dir outside the repo.
4. Write tests for everything.
5. Make sure the full workspace compiles: `cargo check --workspace`
6. Run all tests: `cargo test --workspace`
7. After verified, commit to `dev` and push.
8. NEVER commit any file over 1MB (a pre-commit hook enforces this). This is not a
   suggestion: GitHub's file limit is 100MB and run-capture / JIT-trace dumps
   (e.g. `runs/sh9X-v2boot-*.txt` with JIT_REGION_WATCH/JIT_TRACE on) routinely hit
   150MB+ and block every push. Keep repro/probe logs you commit small (<1MB); for
   large captures, save them to disk but `git rm --cached` / `.gitignore` them so
   they never enter a commit.

## Merge process

Only merge `dev` → `stable` when the user explicitly asks to ship.

## Key people

- Repo: github.com/glm-5-turbo/open-sober
- License: MIT