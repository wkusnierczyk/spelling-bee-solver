# CLAUDE.md

## General Workflow Rules

### Planning

Start every significant task with a planning phase. Discuss the plan with the user before any implementation begins. Once the plan is agreed upon, generate milestones and populate them with issues. Use your judgment on milestone granularity — ask the user when unsure.

### Issues

Each issue must have a meaningful title prefixed with one of `[FEATURE]`, `[BUG]`, or `[CLEANUP]`. During the planning phase, all issues are typically feature issues. Each issue should be assigned to the user, labeled according to its type (`enhancement`, `bug`, or `cleanup`), and added to the repository's project.

### Branching

When working on an issue, create a branch named `{issue-number}-{issue-title-in-lowercase-with-dashes}`. All commits for that issue go into the issue branch. Use sub-branches when needed for independent workstreams, merging them back into the issue branch when done — but do not enforce this as a rule.

Never push or merge directly to main. All changes reach main exclusively through squash-merged pull requests approved by the user.

### Commits

Make atomic or small commits. Do not accumulate work into large, difficult-to-understand commits that bundle everything at once.

Never amend a commit that has already been pushed. Create a new commit instead. Branches are squash-merged to main, so branch history does not matter.

### Code Style

Use full, informative variable names within reasonable boundaries (roughly 20 characters as a soft upper threshold). Prefer `dependency` over `dep`, `property` over `prop`, `validator` over `val`, and so on. Abbreviations are acceptable only when the short form is the universally understood term (e.g., `url`, `id`, `io`).

### Testing

Logic code must be thoroughly tested, with coverage as close to 100% as practical. When evaluating coverage, be explicit about what is not covered by design (e.g., CSS styling, trivial wiring) versus by oversight.

Tests must always exercise both common and corner cases. Critically review every test to verify it asserts both expected inclusions and expected exclusions. When an issue description provides concrete examples (input/output pairs, expected behavior, edge cases), tests must fully and correctly cover every one of those examples — not paraphrased approximations, but the exact scenarios described. Missing a documented example in the test suite is a defect.

Write tests before production code when the interface is well understood. During exploratory phases where the API is still taking shape, postpone tests until the ground is settled — tests should help, not stand in the way. Update tests as needed when the plan or vision changes.

### Delegation

Where relevant, create subagents and delegate smaller chunks of work to them while focusing on the overall flow of the implementation.

### Pull Requests

Push the first bit of work early and create a pull request so that CI workflows start running. As more work is pushed into the PR, keep the title and description in line with the work done. Always include `Fixes #N` (or `Closes #N`) in the PR body so that the issue is automatically closed when the PR is squash-merged.

### CI Monitoring

After each push, monitor the CI workflows. Alert the user if a workflow fails, diagnose the problem, propose a solution, and implement the fix so that the issue does not reappear on the next push.

### Code Review

After finishing work on an issue branch, perform a thorough self-review covering code elegance, efficiency, safety, security, and idiomatic use of each language. Alert the user to any potential issues — whether they warrant immediate fixes or can be deferred to a follow-up.

### Merging

Once all work on an issue is done and the pull request is fully green in CI, ask the user to merge. Never merge to main yourself.

### Audit Trail

Maintain a detailed audit trail in `CLAUDE_AUDIT.md` (untracked — listed in `.gitignore`). Update it on the fly as work proceeds. The audit must log:

- **Issue creation**: issue ID, link, metadata (assignee, labels, project), title, and description.
- **Issue opened for work**: issue ID, title.
- **Branch creation**: issue ID, branch name.
- **Work plan**: the proposed plan, as discussed with the user.
- **Work done**: detailed steps taken, problems encountered, errors, and resolutions.
- **Commits**: commit SHA, branch, issue ID, commit title and body.
- **Pushes**: commit SHA, branch.
- **Pull requests**: PR ID, issue ID, branch, PR title and description.
- **CI workflow status**: which workflows passed, which failed; when failed, include error logs.
- **PR merge**: recorded when announced by the user or discovered via `gh`.
- **User-raised problems**: observed errors, design issues, code feedback, and any other concerns.

### Maintaining This File

Keep `CLAUDE.md` up to date whenever the user provides additional guidance about process, general design direction, or implementation preferences. When the file is updated, also update the corresponding secret gist.

---

## Project: Spelling Bee Solver

### Overview

A Spelling Bee puzzle solver with a Rust backend, React web frontend, React Native Android app, and Kubernetes deployment support.

### Components

| Directory | Stack | Purpose |
| --- | --- | --- |
| `sbs-backend` | Rust, Actix-web | Core solver, dictionary, CLI, REST API (port 8080) |
| `sbs-frontend` | React, TypeScript, Vite | Web UI (port 5173 dev, port 80 in containers) |
| `sbs-mobile` | React Native, Android | Mobile app with offline solving via FFI |
| `sbs-ffi` | Rust cdylib | C-compatible FFI library for mobile |
| `charts/minikube` | Helm | Local Kubernetes deployment |
| `charts/gcp` | Helm | Production GKE deployment |

### API Endpoints

- `POST /solve` — returns word list or validation summary as JSON
- `POST /solve-stream` — Server-Sent Events stream with progress updates
- `GET /health` — health check

Request body: `{"letters": "...", "present": "...", "minimal-word-length": N, "maximal-word-length": N, "validator": "...", "api_key": "..."}`

### FFI Bridge (Rust to Android)

`sbs-ffi` compiles to `libsbs_ffi.so`, wrapped by `sbs-mobile/android/app/src/main/jni/sbs_jni.c` for JNI access. The Kotlin `SbsSolverModule` exposes `solve()` to React Native. Cross-compiled via `cargo-ndk` for `arm64-v8a`, `x86_64`, and `armeabi-v7a`.

### Dictionary

Source: [dwyl/english-words](https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt). Lives at `sbs-backend/data/dictionary.txt`. Downloaded via `make setup-dictionary`. Loaded at runtime via the `SBS_DICT` environment variable (defaults to `data/dictionary.txt`).

### Key Makefile Targets

- `make version-set V=x.y.z` — single source of truth for version across all components
- `make setup-dictionary` — download dictionary
- `make start-local` — run backend + frontend locally
- `make start-compose` / `make stop-compose` — Docker Compose
- `make deploy-minikube` — deploy to local Minikube
- `make deploy-gcp` — full GCP/GKE pipeline
- `make build-android` — cross-compile FFI + JNI for Android
- `make check` — format, lint, and test the backend
