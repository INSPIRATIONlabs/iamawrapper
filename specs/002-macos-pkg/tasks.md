# Tasks: macOS Flat Package (.pkg) Creation

**Input**: Design documents from `/specs/002-macos-pkg/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**TDD Compliance**: Per Constitution Principle I, all phases follow Red-Green-Refactor cycle. Tests are written BEFORE implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Project configuration and dependencies

- [x] T001 Add new dependencies to Cargo.toml (stuckliste, flate2, feature flags)
- [x] T002 [P] Create src/macos/ module directory structure
- [x] T003 [P] Create src/models/macos.rs with MacosPkgRequest and MacosPkgResult structs
- [x] T004 [P] Add macos error variants to src/models/error.rs

---

## Phase 2: Foundational (Core Format Implementations)

**Purpose**: Low-level format implementations that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests First (Red Phase)

> **TDD: Write failing tests BEFORE implementation**

- [x] T005 [P] Write failing unit tests for XAR header serialization in src/macos/xar.rs
- [x] T006 [P] Write failing unit tests for XAR TOC XML generation in src/macos/xar.rs
- [x] T007 [P] Write failing unit tests for XAR archive assembly in src/macos/xar.rs
- [x] T008 [P] Write failing unit tests for CPIO odc format creation in src/macos/cpio.rs
- [x] T009 [P] Write failing unit tests for gzip-compressed payload in src/macos/cpio.rs
- [x] T010 [P] Write failing unit tests for BOM file generation in src/macos/bom.rs
- [x] T011 [P] Write failing unit tests for PackageInfo XML in src/macos/xml.rs
- [x] T012 [P] Write failing unit tests for Distribution XML in src/macos/xml.rs

**Checkpoint**: All tests written and failing (Red)

### XAR Archive Writer Implementation (Green Phase)

- [x] T013 Create XAR header struct and serialization in src/macos/xar.rs (pass T005)
- [x] T014 Implement XAR TOC XML generation using quick-xml in src/macos/xar.rs (pass T006)
- [x] T015 Implement XAR archive assembly (header + TOC + heap) in src/macos/xar.rs (pass T007)
- [x] T016 Add XarBuilder public API with add_file() and finish() methods in src/macos/xar.rs

### CPIO Archive Wrapper Implementation (Green Phase)

- [x] T017 [P] Create CPIO wrapper module with odc format support in src/macos/cpio.rs (pass T008)
- [x] T018 [P] Implement create_payload() function (gzip-compressed cpio) in src/macos/cpio.rs (pass T009)

### BOM File Wrapper Implementation (Green Phase)

- [x] T019 [P] Create BOM wrapper using stuckliste crate in src/macos/bom.rs (pass T010)
- [x] T020 Implement create_bom() function with uid=0, gid=80 in src/macos/bom.rs

### XML Generation Implementation (Green Phase)

- [x] T021 [P] Implement PackageInfo XML generation in src/macos/xml.rs (pass T011)
- [x] T022 [P] Implement Distribution XML generation in src/macos/xml.rs (pass T012)

**Checkpoint**: Foundation ready - all format implementations complete, all tests passing (Green)

---

## Phase 3: User Story 1 - Create Basic macOS Package (Priority: P1) 🎯 MVP

**Goal**: Create valid .pkg files from a source folder with identifier and version

**Independent Test**: Run `iamawrapper macos pkg -c ./test-folder -o ./output --identifier com.test.app --version 1.0.0` and verify the .pkg installs on macOS

### Tests First (Red Phase)

> **TDD: Write failing tests BEFORE implementation**

- [x] T023 [US1] Write failing integration test for basic package creation in tests/macos_test.rs
- [x] T024 [US1] Write failing CLI test for `macos pkg` subcommand in tests/cli_test.rs
- [x] T025 [US1] Write failing test for source folder validation in tests/macos_test.rs
- [x] T026 [US1] Write failing test for progress output in tests/macos_test.rs

**Checkpoint**: US1 tests written and failing (Red)

### CLI Restructure Implementation (Green Phase)

- [x] T027 [US1] Restructure CLI to use subcommands (Commands enum) in src/cli/args.rs
- [x] T028 [US1] Add IntuneCommand subcommand (create/extract) in src/cli/args.rs
- [x] T029 [US1] Add MacosCommand subcommand (pkg) in src/cli/args.rs
- [x] T030 [US1] Add MacosPkgArgs struct with required flags in src/cli/args.rs
- [x] T031 [US1] Update CLI router to dispatch to intune/macos handlers in src/cli/mod.rs (pass T024)

### Core Package Assembly Implementation (Green Phase)

- [x] T032 [US1] Implement payload assembly (collect files → cpio → gzip) in src/macos/payload.rs
- [x] T033 [US1] Implement package() orchestration function in src/macos/mod.rs
- [x] T034 [US1] Wire macos pkg command to package() function in src/cli/mod.rs (pass T023)

### Input Validation Implementation (Green Phase)

- [x] T035 [US1] Add source folder validation (exists, not empty) in src/cli/mod.rs (pass T025)
- [x] T036 [US1] Add identifier format warning (reverse-DNS check) in src/models/macos.rs
- [x] T037 [US1] Add output file exists check with overwrite prompt in src/macos/mod.rs

### Progress and Output Implementation (Green Phase)

- [x] T038 [US1] Add progress bar for file collection in src/cli/mod.rs (pass T026)
- [x] T039 [US1] Add success message with path and size in src/cli/mod.rs
- [x] T040 [US1] Support -q/--quiet and --silent flags in src/cli/mod.rs

### Module Exports

- [x] T041 [US1] Export macos module from src/lib.rs
- [x] T042 [US1] Update Cargo.toml description to include macOS

**Checkpoint**: User Story 1 complete - basic .pkg creation works via CLI, all US1 tests passing (Green)

---

## Phase 4: User Story 2 - Include Installation Scripts (Priority: P2)

**Goal**: Support preinstall and postinstall scripts in packages

**Independent Test**: Create package with `--scripts ./scripts-folder`, install on macOS, verify `/tmp/iamawrapper-install.log` exists

### Tests First (Red Phase)

> **TDD: Write failing tests BEFORE implementation**

- [x] T043 [US2] Write failing integration test for package with scripts in tests/macos_test.rs
- [x] T044 [US2] Write failing unit test for collect_scripts() in src/macos/payload.rs
- [x] T045 [US2] Write failing unit test for auto-set execute permission in src/macos/payload.rs

**Checkpoint**: US2 tests written and failing (Red)

### Scripts Support Implementation (Green Phase)

- [x] T046 [US2] Add --scripts flag to MacosPkgArgs in src/cli/args.rs
- [x] T047 [US2] Add scripts_folder to MacosPkgRequest in src/models/macos.rs
- [x] T048 [US2] Implement collect_scripts() function in src/macos/payload.rs (pass T044)
- [x] T049 [US2] Implement create_scripts_archive() (cpio + gzip, mode 0755) in src/macos/payload.rs
- [x] T050 [US2] Update PackageInfo XML to include scripts element in src/macos/xml.rs
- [x] T051 [US2] Integrate scripts into package assembly in src/macos/mod.rs (pass T043)

### Scripts Validation Implementation (Green Phase)

- [x] T052 [US2] Add scripts folder validation (exists if provided) in src/macos/mod.rs
- [x] T053 [US2] Add warning when scripts folder has no preinstall/postinstall in src/macos/mod.rs
- [x] T054 [US2] Auto-set execute permission (mode 0755) on scripts in src/macos/payload.rs (pass T045)

**Checkpoint**: User Story 2 complete - scripts support works, all US2 tests passing (Green)

---

## Phase 5: User Story 3 - Interactive Mode for macOS Packages (Priority: P3)

**Goal**: Guide users through package creation without memorizing CLI flags

**Independent Test**: Run `iamawrapper` with no arguments, select "macOS package", follow prompts to create valid .pkg

### Tests First (Red Phase)

> **TDD: Write failing tests BEFORE implementation**

- [x] T055 [US3] Write failing test for platform selection prompt in src/cli/interactive.rs
- [x] T056 [US3] Write failing test for macOS prompts flow in src/cli/interactive.rs

**Checkpoint**: US3 tests written and failing (Red)

### Interactive Mode Implementation (Green Phase)

- [x] T057 [US3] Add platform selection prompt (Intune vs macOS) to interactive mode in src/cli/interactive.rs (pass T055)
- [x] T058 [US3] Create run_interactive_macos() function in src/cli/interactive.rs
- [x] T059 [US3] Add prompts for source folder, identifier, version in src/cli/interactive.rs (pass T056)
- [x] T060 [US3] Add optional prompts for install location and scripts folder in src/cli/interactive.rs
- [x] T061 [US3] Add package summary and confirmation prompt in src/cli/interactive.rs
- [x] T062 [US3] Wire interactive macOS flow to package() function in src/cli/mod.rs

**Checkpoint**: User Story 3 complete - interactive mode works for macOS, all US3 tests passing (Green)

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Refinements that affect multiple user stories

- [x] T063 Update README.md with macOS packaging examples
- [x] T064 Run cargo fmt and cargo clippy, fix any issues
- [x] T065 Validate package builds on Linux (cross-platform verification)
- [x] T066 [P] Add cross-platform build verification (Linux build verified)
- [x] T067 Refactor: Review code for TDD compliance and refactor while keeping tests green

**Checkpoint**: All polish complete, ready for release

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories proceed in priority order (P1 → P2 → P3)
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Builds on US1 infrastructure but adds independent capability
- **User Story 3 (P3)**: Builds on US1 and US2 CLI infrastructure

### TDD Workflow Within Each Phase

1. **Red**: Write failing tests first (all test tasks in phase)
2. **Green**: Implement minimum code to pass tests
3. **Refactor**: Improve code quality while keeping tests green
4. **Checkpoint**: Verify all tests pass before moving to next phase

### Parallel Opportunities

**Phase 1 (Setup)**:
```
T001 (deps) → then T002, T003, T004 in parallel
```

**Phase 2 (Foundational) - Tests**:
```
T005-T012 can all run in parallel (different test files)
```

**Phase 2 (Foundational) - Implementation**:
```
T013-T016 (XAR) sequential
T017-T018 (CPIO) parallel with XAR
T019-T020 (BOM) parallel with XAR/CPIO
T021-T022 (XML) parallel with others
```

**Phase 3 (US1)**:
```
T023-T026 (tests) in parallel
T027-T031 (CLI) sequential after tests
T032-T034 (assembly) after CLI
T035-T037 (validation) parallel after assembly
T038-T040 (output) after validation
```

---

## Parallel Example: Phase 2 Tests (Red Phase)

```bash
# Launch all format tests in parallel:
Task: "Write failing unit tests for XAR header in tests/unit/xar_test.rs"
Task: "Write failing unit tests for CPIO odc format in tests/unit/cpio_test.rs"
Task: "Write failing unit tests for BOM file in tests/unit/bom_test.rs"
Task: "Write failing unit tests for PackageInfo XML in tests/unit/xml_test.rs"
Task: "Write failing unit tests for Distribution XML in tests/unit/xml_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (Tests → Implementation)
3. Complete Phase 3: User Story 1 (Tests → Implementation)
4. **STOP and VALIDATE**: All tests green, create test package, install on macOS
5. Deploy/release as v0.2.0

### Incremental Delivery

1. Setup + Foundational (with TDD) → Core formats ready
2. Add User Story 1 (with TDD) → Test independently → Release v0.2.0 (MVP!)
3. Add User Story 2 (with TDD) → Test scripts support → Release v0.2.1
4. Add User Story 3 (with TDD) → Test interactive mode → Release v0.3.0

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable on macOS
- **TDD Compliance**: Tests MUST fail before implementation begins
- XAR implementation is custom (~200-300 lines) per research.md
- Use `stuckliste` for BOM, `flate2` for gzip per research.md
- All packages must use uid=0 (root), gid=80 (wheel)
