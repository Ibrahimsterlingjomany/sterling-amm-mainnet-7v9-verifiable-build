## Live Match Trace 2026-04-20

This note captures the exact local source and artifact traced back to the
current live runtime after the 2026-04-20 extend+upgrade sequence.

Program:
- `7v9sLrk92NNLLUfXLJw3o7MycZNvwsTK6kLWfWb8vcVA`

Current live runtime:
- Executable hash: `6f151b050c8e40540466d15ff76c36bc162962e0f52e59256c2ed0847daa3934`
- File SHA256: `787e7955a78fa1afc5638c2419bab8be67a9ab3ceed2e3b0adf863c23026bdc4`
- Data length: `1122096`

Exact local artifact traced to live:
- `/Users/ibrahimjomanysterling/Sterling_Bridge_M3/runtime/activation/probe_project_light/target/deploy/sterling_amm.so`

Matching live dump:
- `/private/tmp/sterling_live_post_upgrade.so`

Traceability result:
- `programs/sterling_amm/Cargo.toml` in this repo was synchronized to the live
  source tree.
- `programs/sterling_amm/src/lib.rs` in this repo was synchronized to the live
  source tree.
- The synchronized files match the live source workspace byte-for-byte.

Important:
- The source workspace that produced the current live runtime is:
  `/Users/ibrahimjomanysterling/Sterling_Bridge_M3/runtime/activation/probe_project_light`
- That workspace is not a Git repository.
- The current on-chain verification proof still points to the older verified
  commit `c40fb266b5d19bc259da56cd2b7274e0a740ca37`.
- Match True cannot be restored for `6f151b05...` until this exact source tree
  is committed and reproduced in Docker/verifiable form.

Next step:
1. Commit the synchronized source state in a Git branch.
2. Rebuild/verifiy this state in Docker until it reproduces
   `6f151b050c8e40540466d15ff76c36bc162962e0f52e59256c2ed0847daa3934`.
3. Refresh the verification proof on-chain to that exact commit.
