# `tangible-objects-cli` - Scriptable CLI Client

CLI client for a running daemon (`crates/cli`, CLI_CLIENT.md) - built on
[`tangible-objects-client`](CLIENT.md); it speaks GraphQL only, so it
requires `--graphql-listen` on the daemon and never touches devices or
files locally.

```bash
tangible-objects-cli [--endpoint http://host:8080/graphql] [--json] <command>
```

Endpoint resolution: `--endpoint` > `TANGIBLE_ENDPOINT` > `client.toml`
> localhost default (shared with the TUI and the client crate).
`--json` emits raw payloads (piping/`jq`) instead of the line format.

## Commands

| Command | Maps to | Purpose |
|---------|---------|---------|
| `devices` | `deviceInfo` | device name + axis calibration |
| `status` | snapshot bundle | session overview: device, recording, metrics summary |
| `metrics [reset]` | `metrics` / `resetMetrics` | full session metrics; `reset` between test series |
| `message [text]` | `message` / `showMessage` | set overlay text; omit to read the current one |
| `tangibles` | `tangibles` | live tracked poses |
| `touches` | `touchPoints` | active contacts |
| `profiles list [--owner o] [--static-only]` | `profiles` | registered + static profiles |
| `profiles show <id>` | `profiles` | one profile by `PatternId` |
| `profiles register <file.toml> [--owner o] [--lifetime l]` | `registerPattern` | returns the typed result union |
| `profiles update <id> <file.toml>` | `updateProfile` | replace geometry, keep the ID |
| `profiles delete <id>` | `unregisterPattern` | remove a registered pattern |
| `profiles unregister-owner <owner>` | `unregisterOwner` | drop every registration of an owner |
| `profiles train <name>` | `trainProfile` | daemon records a new profile |
| `field-sets list` | `fieldSets` | active occupancy field sets |
| `field-sets register <file.toml> [--owner o] [--lifetime l]` | `registerFieldSet` | returns the typed result union |
| `field-sets unregister <name>` | `unregisterFieldSet` | remove a field set |
| `stages register <name>` / `stages unregister <name>` | `registerStage`/`unregisterStage` | explicit pipeline scopes |
| `filter <mode> [--stability n] [--stage s]` | `setFilterMode` | modes: `on` (=guarded), `off`, `immediate`, `gated`, `guarded`, `tangibles-only` |
| `completion [--mode m] [--radius f] [--max-missing n] [--persist b]` | `completion`/`setCompletion` | read or patch the completion config |
| `record status` | `recordingStatus` | active session counters |
| `record start <path> [--label l]` | `startRecording` | `<path>` is on the **daemon** host |
| `record stop` / `record label [l] [--clear]` | `stopRecording`/`setRecordLabel` | session control + re-tagging |
| `analyze recording <path> [--mode candidates\|contacts] [--profiles-dir d]` | `analyzeRecording` | deviation analysis |
| `analyze similarity [--profiles-dir d]` | `analyzeSimilarity` | pairwise profile-collision report |
| `analyze artifact <path>` | `recordingArtifact` | fetch an artifact's content |
| `watch [--kinds a,b] [--pattern-id p] [--set-id s] [--max-hz n] [--fields [set]] [--tangibles]` | `eventStream` | live domain event stream |

`watch` subscribes `eventStream` - `LAGGED` events print drop counts;
recovery is re-reading `status`/`tangibles`
([`../event/DOMAIN_EVENTS.md`](../event/DOMAIN_EVENTS.md)).

## Typical workflows

```bash
# scripted training series on a headless daemon
tangible-objects-cli message "Legen Sie Figur 10 auf"
tangible-objects-cli metrics reset
tangible-objects-cli record start /tmp/run10 --label figur-10
tangible-objects-cli record stop
tangible-objects-cli metrics --json | jq '.patterns'

# register a game stage and scope its policy before profiles land
tangible-objects-cli stages register game
tangible-objects-cli filter guarded --stage game
```
