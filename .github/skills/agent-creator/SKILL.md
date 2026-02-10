---
name: agent-creator
description: Create new custom agents (.agent.md files) for VS Code Copilot. Use when the user wants to create, scaffold, or design a new custom agent with specific roles, tools, handoffs, and instructions. Guides through agent design decisions and produces a complete .agent.md file.
---

# Agent Creator

This skill guides the creation of custom agents (`.agent.md` files) for VS Code Copilot.

## Agent File Location

Agents live in the `agents/` directory of the ClientSafeAI repository (`/Users/ronhouben/code/ksyos/ClientSafeAI/agents/`). The filename becomes the default agent name if no `name` field is specified.

## Agent File Structure

Every `.agent.md` file consists of **YAML frontmatter** + **Markdown body**.

### Frontmatter Fields

```yaml
---
name: AgentName                    # PascalCase display name (optional, defaults to filename)
description: What this agent does  # Shown as placeholder text in chat input (required)
argument-hint: How to invoke       # Hint shown in chat input field (optional)
tools: ['tool1', 'tool2']          # Available tool sets (required)
agents: ['*']                      # Subagents available (optional, * = all)
handoffs:                          # Workflow transitions (optional)
  - label: Button text
    agent: target-agent
    prompt: Context to pass
    send: false                    # false = prefill, true = auto-submit
---
```

### Available Tool Sets

Pick only the tools the agent needs — fewer tools = more focused behavior.

| Tool set | Purpose |
|----------|---------|
| `read` | Read files, search codebase |
| `search` | Search workspace and web |
| `edit` | Create and modify files |
| `execute` | Run terminal commands |
| `web` | Fetch web pages |
| `vscode` | VS Code commands |
| `todo` | Manage task lists |
| `agent` | Invoke subagents |
| `memory` | Store/retrieve memories |
| `atlassian-mcp/*` | Jira & Confluence access |
| `postgresql-database/*` | Database queries |
| `sonarqube/*` | Code quality analysis |

For MCP servers, use `server-name/*` to include all tools, or `server-name/toolName` for specific tools.

### Advanced Frontmatter Options

| Field | Purpose |
|-------|---------|
| `model` | Lock to a specific AI model (string or prioritized array) |
| `user-invokable` | Set `false` to hide from dropdown (subagent-only) |
| `disable-model-invocation` | Set `true` to prevent other agents from calling this one |
| `target` | `vscode` (default) or `github-copilot` |

## Workflow

When creating an agent, follow these steps:

### 1. Clarify the Agent's Purpose

Determine:
- **Role**: What persona does the agent adopt? (e.g., code reviewer, planner, DBA)
- **Scope**: What should it do and what should it NOT do?
- **Read-only vs read-write**: Does it need to modify files or just analyze?

### 2. Select Tools

Choose the minimal set of tools needed. Guidelines:
- **Planning/analysis agents**: `['read', 'search', 'web', 'todo']` — no edit tools
- **Implementation agents**: `['read', 'search', 'edit', 'execute', 'todo']`
- **Review agents**: `['read', 'search', 'todo']` — read-only
- **Database agents**: `['search', 'execute', 'postgresql-database/*', 'todo']`

### 3. Design Handoffs

Handoffs create guided workflows between agents. Consider:
- What comes **after** this agent finishes its work?
- `send: false` lets the user review before submitting (preferred for most cases)
- `send: true` auto-submits for seamless pipeline transitions

### 4. Write the Body Instructions

The body should include:

1. **Role statement** — One sentence defining who the agent is
2. **Must-follow rules** — Non-negotiable constraints (e.g., "never modify files", "always use todo list")
3. **Domain knowledge** — Key patterns, conventions, or schemas the agent needs
4. **Workflow** — Step-by-step process the agent follows
5. **Output expectations** — What the agent should produce

Keep instructions concise and specific. The agent is already smart — only add what it can't infer.

## Body Writing Guidelines

- Start with a clear role definition: "You are a [role] agent for ClientSafeWeb."
- Use Markdown headers to organize sections
- Reference `.github/copilot-instructions.md` for project conventions when relevant
- Use the `#tool:<tool-name>` syntax to reference specific tools in instructions
- Keep the body under 200 lines — long instructions dilute focus
- Include stopping rules for agents that should NOT do certain things (e.g., planners should not implement)

## Examples

### Minimal Read-Only Agent

```markdown
---
description: Review code changes for security vulnerabilities
tools: ['read', 'search', 'todo']
---

You are a security review agent for ClientSafeWeb.

# Rules
- Never modify files. Your role is analysis only.
- Focus on OWASP Top 10 vulnerabilities.
- Report findings with severity, file location, and remediation steps.
```

### Agent with Handoffs

```markdown
---
name: Planner
description: Create implementation plans
tools: ['read', 'search', 'web', 'todo', 'agent']
handoffs:
  - label: Start Implementation
    agent: Coder
    prompt: Implement the plan outlined above.
    send: false
  - label: Add Tests
    agent: TestWriter
    prompt: Write tests for the plan outlined above.
    send: false
---

You are a planning agent. Research the codebase and produce actionable plans.
Never implement changes yourself.
```

## Naming Conventions

- **Filename**: `PascalCase.agent.md` for role-based agents (e.g., `Reviewer.agent.md`), `kebab-case.agent.md` for task-based agents (e.g., `code-review.agent.md`)
- **name field**: PascalCase, matching the intended display name
- **description**: Start with a verb or noun phrase describing the agent's primary function
