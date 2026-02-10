```prompt
---
agent: Coder
description: Create a new custom agent (.agent.md file) with tools, handoffs, and instructions
tools: ['read', 'search', 'edit', 'todo']
---

# Create a custom agent

Create a new custom agent following the `agent-creator` skill.

Ask for the following if not provided:
- Agent name and purpose
- Read-only or write agent
- Which tools it needs
- Which agents it should hand off to

## Steps

1. Load the `agent-creator` skill
2. Review existing agents in the agents directory for conventions
3. Create the `.agent.md` file with proper YAML frontmatter
4. Add instructions body following the skill's guidelines
5. Update `agent-workflow.instructions.md` with the new agent

```
