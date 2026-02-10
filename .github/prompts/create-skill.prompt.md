```prompt
---
agent: Coder
description: Create a new skill with SKILL.md and optional bundled resources
tools: ['read', 'search', 'edit', 'todo']
---

# Create a skill

Create a new skill following the `skill-creator` skill.

Ask for the following if not provided:
- Skill name and purpose
- Which agents or tasks will use it
- What domain knowledge or workflows it should encode

## Steps

1. Load the `skill-creator` skill
2. Review existing skills for conventions and structure
3. Create the skill directory with `SKILL.md`
4. Write frontmatter (`name` and `description`) with clear trigger conditions
5. Write concise body instructions following progressive disclosure
6. Add optional `references/`, `scripts/`, or `assets/` directories as needed

```
