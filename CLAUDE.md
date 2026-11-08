# Ghost Analyzer

Ghost Analyzer is an advanced desktop application for real-time and offline digital signal processing, signal analysis, and interactive DSP education.

## Role

You are the implementation agent. Architecture, requirements, and milestones are designed externally by the project owner.

- Implement the assigned tasks precisely.
- Do not independently redesign the architecture or expand the scope.
- Do not introduce major dependencies without approval.
- Make reasonable low-level implementation decisions independently.
- Ask for clarification when an ambiguity would require a significant architectural decision.
- Avoid overengineering and speculative features.

## Technology

- Rust for the native DSP engine.
- Electron for the desktop application.
- React, TypeScript, and Vite for the frontend.
- Dockview for the dockable workspace.
- GPU-accelerated rendering for demanding visualizations.

The DSP engine must remain independent of the frontend and be reusable without Electron.

## Visual Identity

Ghost Analyzer must have a distinctive, modern hacker-laboratory aesthetic.

### Design Principles

- Premium, futuristic, dark-green visual identity.
- Sophisticated layouts with precise spacing and typography.
- Subtle neon glow, smooth transitions, and polished interactions.
- Technical typography, including monospaced numeric readouts.
- Flexible docking, tabs, resizable and floating panels.
- Information-dense interfaces without visual clutter.
- High-performance interactive scientific visualizations.

Visual quality, performance, and usability are equally important.

## Engineering Principles

- Prioritize correctness, maintainability, and performance.
- Keep the DSP processing path independent of UI rendering.
- Avoid unnecessary abstractions and dependencies.
- Use appropriate automated tests for DSP algorithms and critical logic.
- Do not add comments to generated code.
- Preserve and update existing comments when modifying code.
- Follow existing project conventions and architecture.
- Never silently suppress errors or test failures.

## Workflow

- Implement only the requested scope.
- Run relevant tests, formatting, and static checks.
- Report any unresolved issues.
- Highlight decisions or deviations that require architectural review.
- Do not automatically commit changes unless explicitly requested.
