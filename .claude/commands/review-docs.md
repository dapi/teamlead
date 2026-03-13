---
name: review-docs
description: Запустить параллельное ревью документации через Agent Team (gaps, completeness, contradictions, consistency)
allowed-tools: Agent, TeamCreate, TeamDelete, TaskCreate, TaskUpdate, TaskList, TaskGet, SendMessage, Read, Glob, Grep
---

# /review-docs

Запусти параллельное ревью документации используя скил `doc-review-team`.

Targets для проверки: $ARGUMENTS

Если targets не указаны — используй дефолтные: `README.md`, `docs/`.

Следуй workflow из скила `doc-review-team` строго по шагам.
