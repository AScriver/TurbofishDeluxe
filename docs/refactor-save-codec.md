# Current-format session codec refactor

Actionable #299 is the bounded codec lane in the [E124 contract](refactor-batch-298-302.md).
The source baseline is `a55f50afcb272ace4fa1cb775c4c85ec71d7bd88`, with a clean
worker checkout before edits. This report describes original project persistence
code; it makes no recovered retail-behavior or gameplay claim.

## Admission decision

Standalone envelopes remain format37; profile managers remain format3. A shared
crate-local session decoder checks the current raw fields before Serde can supply
defaults, decodes the session directly, and calls `AdventureSession::validate`.
Profile entries use that admission during deserialization instead of wrapping
their sessions in an envelope, serializing bytes, decoding a second session and
discarding it. Existing profile current-owner fallback and final manager
validation remain in place. Public CLI entry points and serialized fields stay
compatible, including saver generation, claim and consumed-scene metadata.

Historical envelopes remain rejected. Their unreachable migration branches and
the incomplete diagnostic label table are removed from the current admission
path. Current missing-field failures must return ordinary errors. The checkpoint
writer, publication retries and synchronous durability are outside this lane.

## Tracking limitation

The initial scoped mine list was empty and the available list showed Ready
version6. Claiming returned `INTERNAL_ERROR`, retryMode `never`, correlation
`3ca3c0ae-3d31-4f9c-8a0d-06a4494a532b`. Scoped readback confirmed Ready version6,
unclaimed. Root identified the shared preparation-activity parsing failure and
authorized the installed workflow's required-tool-unavailable fallback. No claim
retry, invented transition, database/service change or completion is made.

## Verification and handoff

The production callers are integrated: `decode_save` admits the standalone
envelope, `load_session` reads it without a migration/publication branch,
`load_profiles` reuses the already parsed current envelope during adoption, and
`ProfileEntry` deserializes its session through `decode_current_session`.
`Profiles::decode` retains the existing current-owner fallback and manager
validation. No codec consumer join is pending. Root owns the later writer
compatibility re-export/profile call-site join, central records and verification
with the other E124 contributions.

On2026-10-11 America/Phoenix, **18 focused regressions passed**. The first
compilation failed with `E0618` and `E0308` because removing historical boolean
branches left an omitted operator and an accidental closure; both expressions
were corrected. The second compilation and all subsequent gates passed. No test
failed. The identity helper initially assumed an absent `rust-toolchain.toml`;
its file list was corrected to the actual Cargo files, while recording the
actual compiler identity. Failed receipts remain private and preserved.

### Exact commands and results

All Cargo compilation, test and lint commands used `--locked --offline`, package
`turbofish-deluxe`, a private target, two build jobs, incremental disabled and
dev/test opt2/debug0 profiles. The actual compiler was Rust1.95.0
`59807616e1fa2540724bfbac14d7976d7e4a3860`, MSVC x86_64 Windows. The existing
read-only libopenmpt0.8.9 import library/runtime supplied linking and test loading;
no tools or dependencies were installed. The full environment and Cargo version
are recorded in `.scratch/refactor/299/identity-01.json`.

| Command | Actual result |
|---|---|
| `cargo fmt --all -- --check` | Passed on the checked Rust source |
| `cargo test --locked --offline --package turbofish-deluxe --lib --test persistence --no-run` | Passed after the two expression corrections; the first filtered test command compiled the final test-only fixture correction |
| `cargo test --locked --offline --package turbofish-deluxe --test persistence current_codec:: -- --nocapture` | 6 passed |
| `cargo test --locked --offline --package turbofish-deluxe --lib current_twenty_one_tests:: -- --nocapture` | 2 passed, including both previously panicking current-field cases |
| `cargo test --locked --offline --package turbofish-deluxe --lib profiles::tests:: -- --nocapture` | 2 passed |
| `cargo clippy --locked --offline --package turbofish-deluxe --lib --bin turbofish-deluxe --test persistence -- -D warnings` | Passed |
| `cargo build --locked --offline --package turbofish-deluxe --bin turbofish-deluxe` | Passed; no launch or normal-launch artifact refresh |

Seven additional existing persistence regressions passed individually with
`cargo test --locked --offline --package turbofish-deluxe --test persistence <filter> -- --nocapture`:

| Exact filter | Passed |
|---|---|
| `modern_save_does_not_repair_missing_or_null_state` | 1 |
| `current_time_trial_envelope_and_actor_form_fields_are_explicit` | 1 |
| `current_tank_four_second_rejects_missing_and_impossible_live_state` | 1 |
| `current_walter_state_and_nullable_glove_are_required_in_direct_board_and_envelope` | 1 |
| `current_nostradamus_actor_food_and_inflight_pickup_require_complete_state` | 1 |
| `current_story_earns_browses_and_reopens_without_recrediting_challenge` | 1 |
| `current35_virtual_combat_preserves_owned_fish_and_reopens_active_state` | 1 |

The final existing library check was
`cargo test --locked --offline --package turbofish-deluxe --lib checkpoints_preserve_complete_current_profiles_and_retained_scenes -- --nocapture`
(1 passed). These filters exercise admission, rule continuation and project
transactions, not GUI gameplay. Historical migration tests remain preserved;
no unfiltered historical suite or native acceptance matrix was run.

### Checked identity and limits

The check identity is the baseline commit above plus exactly the three changed
Rust files below. Documentation changes do not change the checked runtime. The
private identity receipt records all log/test-artifact digests and confirms the
CLI writer tail and profile load/save/saver transaction tail exactly match the
baseline source. Tests independently require complete saved-state equality,
unchanged saver metadata/current-owner fallback, rejection of malformed and
pre37 input, and byte conservation across load/adoption. They do not establish
retail fidelity or full gameplay acceptance.

| Input/artifact | SHA-256 |
|---|---|
| `src/cli.rs` | `AEA30D7C1849C8F5A44D07AB28C39CE2DE3DBE0D5D6DC3530406433BCF9A146B` |
| `src/profiles.rs` | `6DA1509DF6F8AD87DB999A746CD8A278AB0716A385EDEF4CABB9CDD76F6509A9` |
| `tests/persistence.rs` | `D999D3FCF71C4FBE7F2AD5798EE8CADEE23A0D17FE37B2752DA20DD6FC09870B` |
| `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Private normal-bin EXE | `729E8F99ECC1146A229A55B1FC94AC3228F7E0E5A219A82306CED8064F696437` |

Raw receipts and build artifacts remain in the worker's ignored
`.scratch/refactor/299` tree. The disposable load/adoption fixture was removed;
identity readback confirmed its fixture directory empty. All compiler/test
processes completed. No native game launch, original installation, human
process, live/frozen/earned save, service, database or paused automation was
accessed for testing. Actionables readback remains **Ready version6, unclaimed**
under the documented tracking failure; engineering verification is separate from
a tracker completion claim.

### Reconciled contribution

The owned implementation is `d7518654c430e8e67cc2d5a33b7ba26f1e7a995d`.
Main's other four lane contributions and central tracking notes were fetched and
merged cleanly as `0afc4a2e8bf4047a4c5390e079beeff15c55f76b`. The three owned
Rust source digests above remain unchanged. On that reconciled tree,
`cargo fmt --all -- --check` and
`cargo check --locked --offline --package turbofish-deluxe --lib --bin turbofish-deluxe --test persistence`
both passed. No unaffected test filter was repeated. This is compile support for
the reconciled contribution before root's shared joins; it does not replace the
identified earlier EXE or claim the final joined batch is verified.

## Dead legacy-save cleanup

On 2026-10-12 the user explicitly requested removal of dead migration
scaffolding and updates to its tests. This follow-up starts at clean main
`401da2161fa63e42cbb8d5789c74aa49ebf4a897`. A bounded source review mapped
the test consumers; root owns the joined change and validation.

Nine unused reconstruction functions, `MigrationError`, and the production
`LegacyProjectSave` wrapper are removed from Adventure, simulation, invasion
and CLI code. Twelve historical acceptance tests now exercise complete current
sessions, preserving next-tick equality, pending rewards, selectors, earned
state, byte conservation and required-field/corpse-kind rejection. The two
affected library tests retain normal completion and Stinky spawn/serialization
checks. The obsolete historical-envelope fixture helper is removed.

Standalone format37, profile format3 and every serialized field remain.
`StinkyOrigin::LegacyV2Resume` and all three historical `InvasionOrigin`
tags remain readable metadata in current saves. New standalone/profile
round-trip coverage explicitly checks them. The existing codec suite retains
rejection of every pre37 version, malformed input and unchanged saver metadata.
Profile-manager, completed screensaver, normal-launch and native-host sources
are unchanged. No save migration, schema bump or gameplay change is introduced.

### Checks and actual failures

The original `legacy_board_save_migrates_without_losing_next_tick` test fails
against the baseline with `Project saves before format thirty-seven lack
retained mode scenes`, confirming its obsolete expectation. The first changed
test compilation fails with `E0382`: serialization moved `resumed` before its
remaining assertions. Root changes that call to borrow; final checks pass.
Both failed logs remain in ignored `.scratch/legacy-save-cleanup`.

All commands use Rust/Cargo1.95.0, default features, MSVC x86_64, two jobs,
incremental disabled and unoptimized debug/test profiles with debug info
disabled. Cached libopenmpt0.8.9 supplies linking/loading; no installation,
download or normal-launch staging occurs. Cargo targets and logs are isolated
under `.scratch/legacy-save-cleanup`.

| Executed command | Final result |
|---|---|
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings` | Passed |
| `cargo test --locked --offline -p turbofish-deluxe --test persistence current_codec:: -- --nocapture` | 7 passed |
| `cargo test --locked --offline -p turbofish-deluxe --test persistence <filter> -- --exact --nocapture`, each filter below | 12 passed |
| `cargo test --locked --offline -p turbofish-deluxe --lib adventure::tests::first_board_completion_hatches_once_and_invalid_board_is_rejected -- --exact --nocapture` | 1 passed |
| `cargo test --locked --offline -p turbofish-deluxe --lib sim::tests::second_stage_stinky_has_live_spawn_state_and_persisted_origin -- --exact --nocapture` | 1 passed |
| `cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe` | Passed, isolated supporting build |

The twelve exact persistence filters:

```text
current_nimbus_hatch_enters_four_two_and_amp_selector_resumes
current_board_null_gash_selector_enters_four_four
current_fifth_board_reopens_without_losing_earned_state
current_board_save_preserves_next_tick
current_completed_board_keeps_one_pending_reward
current_hatch_resumes_into_level_two_without_duplicate_reward
current_load_preserves_board_state_and_save_bytes
current_stage_three_preserves_progress_growth_gate_and_invasion
current_requires_pet_fields_and_preserves_resumable_selection
current_fourth_board_preserves_earned_state_and_stage_support
current_strong_corpse_kind_round_trips_and_missing_kind_rejects
current_missing_pet_and_score_state_are_rejected
```

Final supporting EXE SHA-256:
`afc8df0125cc8ea76170f9be647a40a804784bfa301e98319c4fb9a057135a11`.
The post-check 122-input source snapshot SHA-256 is
`d2832d1b752acb2dd64abfd58cc2f9d6daec944dd56dff9a5c4641c849c84718`;
only the four production files and persistence tests changed among build inputs.
Exact source/configuration identity is retained in
`.scratch/legacy-save-cleanup/final-identity.local.json`.

Total: 21 focused tests passed. Removed-helper reference, protected-source,
workflow and whitespace checks passed. Test-created load fixtures were removed
by their successful checks. No live/original saves, human game process or
normal-launch artifacts were accessed or changed. Native gameplay, retail
comparison and the separate guppy motion/pose follow-up remain outside this
cleanup. No work item was supplied and this chat had no owned claim;
Actionables was not updated. Concurrent follow-up documentation is preserved
and excluded from this cleanup commit.
