# Turbofish Deluxe

Turbofish Deluxe is a standalone Rust reimplementation of Insaniquarium Deluxe for Windows. It runs its own simulation and loads assets from the player's installed copy of the game.

A legitimate copy of [Insaniquarium Deluxe](https://store.steampowered.com/app/3320/Insaniquarium_Deluxe/) is required. Game binaries and assets are not included. This is an unofficial project, unaffiliated with the original developers or publisher.

The runtime includes Adventure, Time Trial, Challenge, Virtual Tank, Sandbox, profiles, menus and audio. Development is ongoing; full gameplay, visual and audio fidelity is not yet verified. See [runtime coverage](docs/compatibility.md) for implementation status and known limitations.

## Requirements

- Windows 11
- Rust 1.95 with the `x86_64-pc-windows-msvc` toolchain and MSVC build tools
- PowerShell 7
- An installed copy of Insaniquarium Deluxe

## Build and run

From the repository root:

```powershell
.\scripts\Start-TurbofishDeluxe.ps1
```

The launcher builds an optimized release, verifies and stages native audio dependencies, and discovers the game through Steam. It resumes the current profile or creates one on first launch.

| Launcher option | Purpose |
|---|---|
| `-Development` | Build and run the debug configuration |
| `-PrepareOnly` | Build and stage dependencies without launching |
| `-Offline` | Use cached dependencies without downloading |
| `-GameArguments` | Pass runtime arguments to the executable |

To specify the game installation:

```powershell
.\scripts\Start-TurbofishDeluxe.ps1 -GameArguments @('--game-dir', '<game-directory>')
```

## Saves and controls

Project data is stored in `%LOCALAPPDATA%\TurbofishDeluxe`, separate from the original game's saves. Use `--save-dir <directory>` to select another location. Only the current project-save format is supported; older formats and original-game saves are not imported.

`--new-game` resets the active session. Use a separate save directory for isolated runs.

| Input | Action |
|---|---|
| Escape | Pause or resume the active tank |
| Menu | Open the mode selector |
| S | Save |
| Q while paused | Save and exit |
| F1 | Open Help |
| Enter | Activate Continue |

## Development

### Focused runtime checks

`Test-Runtime.ps1` runs formatting, strict Clippy, workspace tests and build staging by default. Use `-Check` to run a specific check:

```powershell
.\scripts\Test-Runtime.ps1
.\scripts\Test-Runtime.ps1 -Check Format
.\scripts\Test-Runtime.ps1 -Check Lint
.\scripts\Test-Runtime.ps1 -Check Unit -TestFilter 'bilaterus::tests::'
.\scripts\Test-Runtime.ps1 -Check Persistence
.\scripts\Test-Runtime.ps1 -Check Build
```

Additional checks are `Tests`, `Assets`, `Fonts`, `Binding` and `OwnedMusic`. Test-name filtering is supported by individual test suites. Installed music-decoder tests require `-GameDirectory` with `All` or `OwnedMusic`.

### Workflow checks

```powershell
.\scripts\Test-Workflow.ps1
```

Validates documentation links, heading anchors, whitespace and repository tracking rules. For configured Codex hosts, see [PowerShell validation](docs/playtest-helpers.md#run-and-validate).

### Accelerated gameplay tests

`--test-speed <1..8>` enables accelerated diagnostic runs. Speeds above 1x require separate save and evidence directories, `--mute` and a positive `--quit-after` limit. Accelerated runs do not establish normal-speed fidelity.

Native reproduction procedures are documented in [playtest helpers](docs/playtest-helpers.md).

## Documentation

| Document | Contents |
|---|---|
| [Requirements](docs/requirements.md) | Product scope and constraints |
| [Runtime coverage](docs/compatibility.md) | Implemented systems and remaining limitations |
| [Design](docs/DESIGN.md) | Architecture and behavior contracts |
| [Status](STATUS.md) | Current development state |
| [Change log](MODLOG.md) | Implementation and verification history |
| [Audio](docs/audio.md) | Native dependencies and audio integration |
| [Evidence tooling](docs/evidence-tooling.md) | Build identities and diagnostic records |
| [Screensaver](docs/screensaver.md) | Existing host functionality and limitations |
| [Agent workflow](AGENTS.md) | Contributor automation instructions |

## Licensing

The runtime-code license and distribution plan remain undecided. See [provenance](docs/provenance.md) and [third-party notices](THIRD-PARTY-NOTICES.md) for source attribution and dependency licenses.

Machine-local files, reference material and extracted assets belong in the ignored `local/`, `.scratch/` and `private/` directories.
