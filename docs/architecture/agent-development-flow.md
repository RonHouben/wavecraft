# Agent Development Flow

This project uses specialized agents with distinct responsibilities that hand off to each other throughout the development lifecycle.

## Agent Roles

| Agent | Role | Key Outputs |
|-------|------|-------------|
| **PO** (Product Owner) | Owns product vision, roadmap, and feature prioritization | User stories, `docs/roadmap.md` |
| **Architect** | Designs system architecture, enforces technical constraints | Low-level designs in `docs/feature-specs/{feature}/` |
| **Planner** | Creates detailed implementation plans | `docs/feature-specs/{feature}/implementation-plan.md` |
| **Coder** | Implements features, writes production code | Code changes, PRs |
| **Tester** | Executes manual tests, documents results | `docs/feature-specs/{feature}/test-plan.md` |
| **QA** | Static analysis, code quality verification | QA reports |

## Standard Feature Development Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        FEATURE DEVELOPMENT FLOW                         │
└─────────────────────────────────────────────────────────────────────────┘

  ┌──────┐                                                                 
  │  PO  │  Feature Request / User Story                                   
  └──┬───┘                                                                 
     │                                                                     
     │  "Create low level design"                                          
     ▼                                                                     
┌──────────┐                                                               
│ Architect│  Low-Level Design                                             
└────┬─────┘  └─► docs/feature-specs/{feature}/low-level-design-{feature}.md
     │                                                                     
     │  "Create implementation plan"                                       
     ▼                                                                     
┌─────────┐                                                                
│ Planner │  Implementation Plan                                           
└────┬────┘  └─► docs/feature-specs/{feature}/implementation-plan.md       
     │                                                                     
     │  "Start Implementation"                                             
     ▼                                                                     
┌────────┐                                                                 
│ Coder  │  Implementation + PR                                            
└────┬───┘  └─► docs/feature-specs/{feature}/implementation-progress.md    
     │                                                                     
     │  "Test Implementation"                                              
     ▼                                                                     
┌────────┐◄─────────────────────────────────────────────────────┐          
│ Tester │  Manual Testing                                      │          
└────┬───┘  └─► docs/feature-specs/{feature}/test-plan.md       │          
     │                                                          │          
     │ ─────────────────────────────────────────────┐           │          
     │  Test Issues Found?                          │ No Issues │          
     ▼                                              ▼           │          
┌────────┐                                    ┌──────┐          │          
│ Coder  │  Fix Test Issues ──► Re-test       │  QA  │  Static  │          
└────────┘                                    └──┬───┘  Analysis│          
                                                 │     └─► QA-report.md    
                                   ┌─────────────┴─────────────┐           
                                   │  QA Issues Found?         │ No Issues 
                                   ▼                           ▼           
                             ┌────────┐                  ┌──────────┐      
                             │ Coder  │  Fix Findings    │ Architect│      
                             └────┬───┘                  └────┬─────┘      
                                  │                           │ Update     
                                  └──────► Re-test ───────────┤ Arch Docs  
                                                              ▼            
                                                        ┌──────┐           
                                                        │  PO  │  Update   
                                                        └──┬───┘  Roadmap  
                                                           │      Archive  
                                                           ▼               
                                                    ✅ Feature Complete    
```

## Handoff Summary

| From | To | Trigger | What Gets Passed |
|------|----|---------|------------------|
| PO → Architect | "Create low level design" | Feature requirements, user stories |
| Architect → Planner | "Create implementation plan" | Low-level design document |
| Planner → Coder | "Start Implementation" | Implementation plan |
| Coder → Tester | "Test Implementation" | Completed implementation |
| Tester → Coder | "Fix Issues" | Test failures documented in test-plan.md |
| Tester → QA | "Run QA" | All tests passing |
| QA → Coder | "Fix findings" | QA report with severity/location |
| Coder → Tester | "Re-test" | QA findings fixed |
| QA → Architect | "Update architectural Docs" | No QA issues, implementation review |
| Architect → PO | "Update roadmap" | Architecture docs updated |

## Key Documentation Artifacts

All feature documentation lives in `docs/feature-specs/{feature}/`:

```
docs/feature-specs/{feature}/
├── user-stories.md              # PO: User requirements
├── low-level-design-{feature}.md # Architect: Technical design
├── implementation-plan.md       # Planner: Step-by-step plan
├── implementation-progress.md   # Coder: Progress tracking
├── test-plan.md                 # Tester: Test cases & results
└── QA-report.md                 # QA: Quality review & findings
```

On completion, PO archives the entire feature folder to `docs/feature-specs/_archive/{feature}/`.

## Agent Constraints

| Agent | Can Edit Code? | Can Edit Roadmap? | Can Edit Archived Specs? |
|-------|----------------|-------------------|--------------------------|
| PO | ❌ | ✅ (exclusive) | ❌ |
| Architect | ❌ | ❌ | ❌ |
| Planner | ❌ | ❌ | ❌ |
| Coder | ✅ | ❌ | ❌ |
| Tester | ❌ | ❌ | ❌ |
| QA | ❌ | ❌ | ❌ |

## When to Invoke Each Agent

- **Start with PO** when: New feature request, prioritization question, roadmap update needed
- **Use Architect** when: Design decisions needed, architectural review, defining boundaries
- **Use Planner** when: Complex feature needs breakdown, multi-step implementation
- **Use Coder** when: Ready to implement, bug fixes, code changes
- **Use Tester** when: Feature ready for manual testing, creating test plans
- **Use QA** when: Code review needed, static analysis, quality verification
