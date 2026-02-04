---
applyTo: "**"
---

# MOST Important guidelines
The following are the MOST important guidelines to follow when editing files in this repository:
- never update files under the `/docs/feature-specs/_archive/` directory. They are kept for historical reference only.
- ONLY the Product Owner agent is allowed to edit the roadmap file located at `/docs/roadmap.md`! When any other agent needs changes to the roadmap, they must hand off to the Product Owner agent.
- Before making changes in the code, check the #file:../docs/architecture/coding-standards.md file for relevant coding standards and conventions.
- For understanding the overall project architecture, SDK structure, and design decisions, refer to #file:../docs/architecture/high-level-design.md document.
- For understanding the audio input via WASM architecture, refer to #file:../docs/feature-specs/audio-input-via-wasm/high-level-design.md for the design overview, tiered backend system, and parameter ownership model.
- Always keep the #tool:todo  list up to date with relevant tasks and their statuses.

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