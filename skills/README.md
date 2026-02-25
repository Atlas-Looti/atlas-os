# Atlas OS — Skills

Pre-built [OpenClaw](https://github.com/openclaw/openclaw) skills for operating Atlas OS via AI agents.

## Available Skills

| Skill | Description |
|---|---|
| [`atlas-os`](./atlas-os/) | Full command reference, JSON schemas, trading workflows, and troubleshooting for the Atlas CLI |

## Usage

### With OpenClaw

Copy the skill folder into your OpenClaw workspace:

```bash
cp -r skills/atlas-os ~/.openclaw/workspace/skills/
```

Or install the packaged `.skill` file if available.

### As Reference

Each skill's `SKILL.md` contains everything an AI agent needs to operate Atlas OS:

- **Command reference** — all 50+ CLI commands with syntax and examples
- **JSON output schemas** — exact response format for every command
- **NDJSON streaming events** — real-time data format
- **Trading workflows** — step-by-step agent playbooks
- **Error codes & recovery** — structured error handling
- **Configuration** — full config schema and keys

## Structure

```
skills/
└── atlas-os/
    ├── SKILL.md                    # Main skill (command ref + config + troubleshooting)
    └── references/
        ├── json-schemas.md         # All JSON output schemas + error codes
        └── workflows.md           # 10 agent trading workflows + error recovery
```

## Creating New Skills

See [OpenClaw Skill Creator](https://github.com/openclaw/openclaw) for the skill specification.
