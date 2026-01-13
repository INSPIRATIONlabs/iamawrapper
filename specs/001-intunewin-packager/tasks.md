# Tasks: Cross-Platform IntuneWin Packager

**Input**: Design documents from `/specs/001-intunewin-packager/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Included per constitution requirement (Test-First Development is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root (per plan.md)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Rust project initialization and basic structure

- [x] T001 Initialize Rust project with `cargo init --name intunewin` at repository root
- [x] T002 Configure Cargo.toml with all dependencies: clap, zip, aes, cbc, hmac, sha2, base64, quick-xml, indicatif, dialoguer, rand, thiserror, anyhow
- [x] T003 [P] Create directory structure: src/cli/, src/packager/, src/models/
- [x] T004 [P] Configure clippy.toml and rustfmt.toml for code quality
- [x] T005 [P] Create tests/ directory structure: tests/unit/, tests/integration/, tests/contract/, tests/contract/fixtures/
- [x] T006 Create src/lib.rs with module declarations for all submodules
- [x] T007 Create .github/workflows/ci.yml for cross-platform build and test

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T008 Create PackageError enum with all error variants in src/models/error.rs
- [x] T009 [P] Create Verbosity enum in src/models/mod.rs
- [x] T010 [P] Create SourceFile struct in src/models/package.rs
- [x] T011 Create SourcePackage struct with file collection logic in src/models/package.rs
- [x] T012 [P] Create EncryptionInfo struct in src/models/detection.rs
- [x] T013 Create DetectionMetadata struct in src/models/detection.rs
- [x] T014 Create IntuneWinPackage struct in src/models/package.rs
- [x] T015 Create PackageRequest struct with validation in src/models/package.rs
- [x] T016 Configure exit code constants matching CLI contract in src/models/error.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Package Application for Intune (Priority: P1)

**Goal**: Core packaging functionality - create valid .intunewin files from source folders

**Independent Test**: Provide a source folder with an MSI/EXE and verify the output .intunewin file structure matches the file format contract

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T017 [P] [US1] Contract test for Detection.xml format in tests/contract/detection_xml_test.rs
- [x] T018 [P] [US1] Contract test for encrypted file structure (HMAC+IV+ciphertext) in tests/contract/encryption_test.rs
- [x] T019 [P] [US1] Contract test for outer ZIP structure in tests/contract/package_structure_test.rs
- [x] T020 [P] [US1] Unit test for AES-256-CBC encryption in tests/unit/encrypt_test.rs
- [x] T021 [P] [US1] Unit test for HMAC-SHA256 computation in tests/unit/encrypt_test.rs
- [x] T022 [P] [US1] Unit test for ZIP archive creation in tests/unit/archive_test.rs
- [x] T023 [P] [US1] Unit test for Detection.xml generation in tests/unit/metadata_test.rs
- [x] T024 [US1] Integration test for full packaging workflow in tests/integration/package_test.rs

### Implementation for User Story 1

- [x] T025 [P] [US1] Implement random key generation (encryption key, MAC key, IV) in src/packager/encrypt.rs
- [x] T026 [P] [US1] Implement AES-256-CBC encryption with PKCS7 padding in src/packager/encrypt.rs
- [x] T027 [US1] Implement HMAC-SHA256 computation over (IV || ciphertext) in src/packager/encrypt.rs
- [x] T028 [US1] Implement encrypted file assembly (HMAC + IV + ciphertext) in src/packager/encrypt.rs
- [x] T029 [US1] Implement SHA256 file digest computation in src/packager/encrypt.rs
- [x] T030 [P] [US1] Implement inner ZIP archive creation with streaming in src/packager/archive.rs
- [x] T031 [US1] Implement file enumeration including hidden files in src/packager/archive.rs
- [x] T032 [US1] Implement symbolic link following in src/packager/archive.rs
- [x] T033 [US1] Implement Detection.xml generation matching exact format in src/packager/metadata.rs
- [x] T034 [US1] Implement Base64 encoding for all crypto values in src/packager/metadata.rs
- [x] T035 [US1] Implement outer ZIP assembly (IntuneWinPackage structure) in src/packager/mod.rs
- [x] T036 [US1] Implement main package() function orchestrating the workflow in src/packager/mod.rs
- [x] T037 [US1] Implement output file naming (setup file base + .intunewin) in src/packager/mod.rs
- [x] T038 [US1] Add validation for source folder and setup file in src/packager/mod.rs

**Checkpoint**: At this point, User Story 1 should be fully functional - can create valid .intunewin files

---

## Phase 4: User Story 2 - Cross-Platform Packaging (Priority: P1)

**Goal**: Ensure tool works identically on Linux, macOS, Windows (x64 and ARM64)

**Independent Test**: Run the same packaging operation on all 6 platforms and verify output compatibility

### Tests for User Story 2

- [x] T039 [P] [US2] Unit test for cross-platform path handling in tests/unit/archive_test.rs
- [x] T040 [P] [US2] Unit test for hidden file detection across platforms in tests/unit/archive_test.rs
- [x] T041 [US2] Integration test validating ZIP uses forward slashes in tests/integration/package_test.rs

### Implementation for User Story 2

- [x] T042 [P] [US2] Ensure path separators are normalized to forward slashes in src/packager/archive.rs
- [x] T043 [US2] Implement cross-platform hidden file inclusion (dotfiles and Windows hidden attribute) in src/packager/archive.rs
- [x] T044 [US2] Configure cross-compilation targets in Cargo.toml for all 6 platforms
- [x] T045 [US2] Update .github/workflows/ci.yml with matrix build for all targets
- [x] T046 [US2] Add release workflow for binary distribution in .github/workflows/release.yml

**Checkpoint**: Tool produces identical output on all platforms

---

## Phase 5: User Story 3 - Command-Line Interface Compatibility (Priority: P2)

**Goal**: CLI compatible with original Microsoft tool flags (-c, -s, -o, -q)

**Independent Test**: Run existing scripts using original tool parameters without modification

### Tests for User Story 3

- [x] T047 [P] [US3] Unit test for CLI argument parsing in tests/unit/args_test.rs
- [x] T048 [P] [US3] Unit test for quiet mode behavior in tests/unit/args_test.rs
- [x] T049 [P] [US3] Unit test for silent mode behavior in tests/unit/args_test.rs
- [x] T050 [US3] Integration test for full CLI workflow in tests/integration/cli_test.rs
- [x] T051 [US3] Integration test for exit codes on error conditions in tests/integration/cli_test.rs

### Implementation for User Story 3

- [x] T052 [P] [US3] Define CliArgs struct with clap derive in src/cli/args.rs
- [x] T053 [US3] Implement -c/--content argument for source folder in src/cli/args.rs
- [x] T054 [US3] Implement -s/--setup argument for setup file in src/cli/args.rs
- [x] T055 [US3] Implement -o/--output argument for output folder in src/cli/args.rs
- [x] T056 [US3] Implement -n/--name optional argument for custom filename in src/cli/args.rs
- [x] T057 [US3] Implement -q/--quiet flag for quiet mode in src/cli/args.rs
- [x] T058 [US3] Implement --silent flag with -qq alias in src/cli/args.rs
- [x] T059 [US3] Implement -h/--help and -v/--version flags in src/cli/args.rs
- [x] T060 [US3] Implement mode detection (CLI vs Interactive) in src/cli/mod.rs
- [x] T061 [US3] Implement progress bar display with indicatif in src/cli/mod.rs
- [x] T062 [US3] Implement verbosity-aware output (Normal/Quiet/Silent) in src/cli/mod.rs
- [x] T063 [US3] Implement exit code mapping from PackageError in src/main.rs
- [x] T064 [US3] Wire CLI to packager in src/main.rs

**Checkpoint**: Tool can be used as drop-in replacement for Microsoft tool

---

## Phase 6: User Story 4 - Interactive Mode (Priority: P3)

**Goal**: Guide new users through packaging with interactive prompts

**Independent Test**: Run tool without arguments and complete packaging via prompts

### Tests for User Story 4

- [x] T065 [P] [US4] Unit test for prompt flow logic in tests/unit/interactive_test.rs
- [x] T066 [US4] Integration test for interactive workflow (mocked stdin) in tests/integration/interactive_test.rs

### Implementation for User Story 4

- [x] T067 [US4] Implement source folder prompt with path validation in src/cli/interactive.rs
- [x] T068 [US4] Implement file listing for setup file selection in src/cli/interactive.rs
- [x] T069 [US4] Implement setup file selection prompt with dialoguer in src/cli/interactive.rs
- [x] T070 [US4] Implement output folder prompt in src/cli/interactive.rs
- [x] T071 [US4] Implement overwrite confirmation prompt in src/cli/interactive.rs
- [x] T072 [US4] Implement packaging summary display in src/cli/interactive.rs
- [x] T073 [US4] Wire interactive mode to main flow in src/cli/mod.rs

**Checkpoint**: All user stories should now be independently functional

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T074 [P] Streaming optimization for large files (up to 8GB) in src/packager/archive.rs
- [x] T075 [P] Memory budget enforcement (<100MB for 8GB sources) in src/packager/mod.rs
- [x] T076 [P] Performance optimization for 100MB in <30s target in src/packager/mod.rs
- [x] T077 [P] Add comprehensive error messages for all error types in src/models/error.rs
- [x] T078 Code cleanup and clippy warning resolution
- [x] T079 Run rustfmt on all source files
- [x] T080 Validate quickstart.md examples work correctly
- [x] T081 [P] Create sample test fixtures in tests/contract/fixtures/
- [x] T082 Final integration test with real-world package scenarios in tests/integration/package_test.rs

---

## Phase 8: Bonus Features (Added Post-MVP)

**Purpose**: Additional features beyond original spec

- [x] T083 [BONUS] Implement --unpack flag for extracting .intunewin files in src/cli/args.rs
- [x] T084 [BONUS] Implement Detection.xml parsing in src/packager/metadata.rs
- [x] T085 [BONUS] Implement AES-256-CBC decryption with HMAC verification in src/packager/encrypt.rs
- [x] T086 [BONUS] Implement unpack() function in src/packager/mod.rs
- [x] T087 [BONUS] Add UnpackRequest and UnpackResult structs in src/models/package.rs
- [x] T088 [BONUS] Add decryption-related error variants in src/models/error.rs
- [x] T089 [BONUS] Fix Detection.xml format to match Microsoft (CRLF, ToolVersion, file order)
- [x] T090 [BONUS] Add LICENSE file
- [x] T091 [BONUS] Create user-friendly README.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (P1) and US2 (P1) have equal priority
  - US3 (P2) and US4 (P3) can proceed after US1 provides core packaging
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

| Story | Priority | Dependencies | Can Start After |
|-------|----------|--------------|-----------------|
| US1 - Core Packaging | P1 | Foundational only | Phase 2 complete |
| US2 - Cross-Platform | P1 | US1 (needs packaging to verify) | T036 complete |
| US3 - CLI Compatibility | P2 | US1 (needs package() function) | T036 complete |
| US4 - Interactive Mode | P3 | US3 (needs CLI infrastructure) | T064 complete |

### Within Each User Story

1. Tests MUST be written and FAIL before implementation
2. Models/structs before functions
3. Core functions before integration
4. Story complete before moving to next priority

### Parallel Opportunities

**Phase 1 - Setup (4 parallel streams)**:
- T003, T004, T005 can run in parallel

**Phase 2 - Foundational (3 parallel streams)**:
- T009, T010, T012 can run in parallel after T008

**Phase 3 - User Story 1 Tests (8 parallel streams)**:
- T017, T018, T019, T020, T021, T022, T023 can all run in parallel

**Phase 3 - User Story 1 Implementation (2 parallel streams)**:
- T025, T026, T030 can run in parallel

---

## Parallel Example: User Story 1 Tests

```bash
# Launch all contract tests for User Story 1 together:
Task: "Contract test for Detection.xml format in tests/contract/detection_xml_test.rs"
Task: "Contract test for encrypted file structure in tests/contract/encryption_test.rs"
Task: "Contract test for outer ZIP structure in tests/contract/package_structure_test.rs"

# Launch all unit tests for User Story 1 together:
Task: "Unit test for AES-256-CBC encryption in tests/unit/encrypt_test.rs"
Task: "Unit test for HMAC-SHA256 computation in tests/unit/encrypt_test.rs"
Task: "Unit test for ZIP archive creation in tests/unit/archive_test.rs"
Task: "Unit test for Detection.xml generation in tests/unit/metadata_test.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: Foundational (T008-T016)
3. Complete Phase 3: User Story 1 (T017-T038)
4. **STOP and VALIDATE**: Create a .intunewin file and verify format
5. This is a functional MVP - can create Intune packages!

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1 → Can create packages (MVP!)
3. Add US2 → Works on all platforms
4. Add US3 → Full CLI compatibility
5. Add US4 → User-friendly interactive mode
6. Polish → Production ready

### Parallel Team Strategy

With multiple developers after Phase 2:

- Developer A: User Story 1 (Core Packaging)
- Developer B: User Story 2 (Cross-Platform) - after A completes T036
- Developer C: User Story 3 (CLI) - after A completes T036
- Developer D: User Story 4 (Interactive) - after C completes T064

---

## Task Summary

| Phase | Tasks | Parallel Tasks | Status |
|-------|-------|----------------|--------|
| Phase 1: Setup | 7 | 3 | ✅ Complete |
| Phase 2: Foundational | 9 | 3 | ✅ Complete |
| Phase 3: US1 - Core Packaging | 22 | 10 | ✅ Complete |
| Phase 4: US2 - Cross-Platform | 8 | 3 | ✅ Complete |
| Phase 5: US3 - CLI Compatibility | 18 | 5 | ✅ Complete |
| Phase 6: US4 - Interactive Mode | 9 | 1 | ✅ Complete |
| Phase 7: Polish | 9 | 5 | ✅ Complete |
| Phase 8: Bonus Features | 9 | 0 | ✅ Complete |
| **Total** | **91** | **30** | ✅ **All Complete** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently completable and testable
- Tests MUST fail before implementing (TDD per constitution)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
