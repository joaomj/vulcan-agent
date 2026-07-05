# Interesting ideas for an AI coding agent

## Skills
- My current skills: `/Users/joao/.config/opencode/skills`
- AI engineering skills:
  - [to-prd](https://www.aihero.dev/skills-to-prd)
  - [grill-with-docs](https://www.aihero.dev/grill-with-docs)
  - [to-issues](https://www.aihero.dev/skills-to-issues)
  - [tdd](https://www.aihero.dev/skills-tdd)
  - [improve-codebase-archtecture](https://www.aihero.dev/skills-improve-codebase-architecture)
  - [engineering](https://github.com/anthropics/knowledge-work-plugins/tree/main/engineering/skills)
  - [data](https://github.com/anthropics/knowledge-work-plugins/tree/main/data/skills)

## Codebase indexing
- Source: https://x.com/max_paperclips/status/2071465351959998723
- Idea: The solution for coding agents having finite context windows was to use grep. But the problem now is they have tunnel vision because they don't understand the whole codebase. Changes in one area of the code base often don't align to the broad project vision. Because grep alone wasn't a good solution. there were plenty of solutions between "passive semantic rag" and "grep the entire codebase". Why not LSPs that warn the model that it's doing it wrong, or hybrid BM25-semantic indices over the codebase, with active tools to look up what it needs, skills acting as a code map & internal docs. there's a lot of stuff you can do. Anthropic sophon-locked everyone by saying "grep is all you need", people just bought into it because hype. grep should be 1 tool it uses among many, not the whole.
- **Cursor** does something like this. The objective is to improve agent awareness and also save costs by avoiding repeatdly querying the codebase. See: 
  - (https://towardsdatascience.com/how-cursor-actually-indexes-your-codebase/)
  - (https://cursor.com/blog/secure-codebase-indexing)
- See also: https://github.com/colbymchenry/codegraph

## Self-improving agent
- Source: https://x.com/vercel_dev/status/2073129220377849968
- Give your agent the ability to introspect its past runs, spot inefficiencies, errors, redundant tool calls, and produce new prompts and skills.
- The first thing is to log all traces and conversation data. This is the raw material used by the agent to learn upon.
- See `Hermes Agent` below.

## Persistent Memory
- Essential to make the AI agent stateful.
- See `Hermes Agent` below.
- See also: https://github.com/tobi/qmd

## Inspiring agents

### [Hermes Agent](https://github.com/NousResearch/hermes-agent#)
- Pros: 
  - closed loop self-improving feature. See:
    - (https://hermes-agent.nousresearch.com/docs/user-guide/features/skills#learning-a-skill-from-sources-learn)
    - (https://hermes-agent.nousresearch.com/docs/user-guide/features/skills#when-the-agent-creates-skills)
  - telegram client
  - loose coupling architecture (https://hermes-agent.nousresearch.com/docs/developer-guide/architecture#design-principles)
  - persistent memory. See:
    - (https://hermes-agent.nousresearch.com/docs/user-guide/features/memory)
    - (https://hermes-agent.nousresearch.com/docs/user-guide/features/honcho)
- Cons:
  - Feature bloat: too many skills, tools, plugins, etc
  - Python
  - High token consumption
  - Rudimentar memory system
- See: https://hermes-agent.nousresearch.com/docs/developer-guide/architecture

### [Pi](https://github.com/earendil-works/pi)
- Pros:
  - very slim
  - very extendable
- Cons:
  - typescript
  - most features non-native (i.e: needs to be built or connected by the user). Its Hermes opposite.
- Docs: https://pi.dev/docs/latest

### [OMP](https://github.com/can1357/oh-my-pi)
- Pros:
  - fork of Pi = very slim
  - context isolation through worktrees: https://omp.sh/docs/sessions
  - native memory = https://omp.sh/docs/memory
  - some experienced engineers are praising it
- Cons: same as Pi =; also, no telegram.

## [Opencode](https://github.com/anomalyco/opencode)
- Pros:
  - biggest community
  - its the agent i use most
  - good balance between features and extendability
  - broadest LLM provider compatibility
- Cons:
  - no native memory system
  - typescript/bun
  - bloated system prompt
  - no telegram client
  - no voice mode/dictation mode
  - fragile session persistence (sqlite files scattered accross the host) and guardrails (an AI agent wiped my session data mid-task because of an auto-update)
  - somewhat slow UI (i use TUI only - i dont have need for an app interface)
