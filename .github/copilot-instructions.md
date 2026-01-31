---
applyTo: "**"
---

# MOST Important guidelines
The following are the MOST important guidelines to follow when editing files in this repository:
- never update files under the `/docs/feature-specs/_archive/` directory. They are kept for historical reference only.
- ONLY the Product Owner agent is allowed to edit the roadmap file located at `/docs/roadmap.md`! When any other agent needs changes to the roadmap, they must hand off to the Product Owner agent.

---

# Agent Development Flow

This project uses specialized agents with distinct responsibilities that hand off to each other throughout the development lifecycle.

**📖 For the complete agent development flow, roles, handoffs, and diagrams, always refer to:**
**[docs/architecture/agent-development-flow.md](../docs/architecture/agent-development-flow.md)**

That document is the single source of truth for:
- Agent roles and responsibilities
- Standard feature development flow diagram
- Handoff triggers between agents
- Key documentation artifacts
- Agent constraints (code editing, roadmap access, etc.)
- When to invoke each agent