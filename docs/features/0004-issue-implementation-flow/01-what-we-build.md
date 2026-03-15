# Feature 0004: Что строим

## Проблема

Сейчас проект умеет довести issue только до `Ready for Implementation`.

После этого нет канонического ответа на вопросы:

- как единый `run <issue>` должен понять, что теперь нужно запускать coding
  stage, а не analysis;
- как approved analysis artifacts становятся обязательным входом реализации;
- как выглядит lifecycle branch/worktree/PR на implementation stage;
- какие quality gates обязательны до code review;
- как отделить review плана от review кода.

## Пользователь

Основной пользователь:

- владелец репозитория;
- оператор, который запускает `run <issue>`;
- агент реализации;
- ревьюер, который принимает код после прохождения CI.

## Результат

Полезным результатом считается implementation flow, в котором:

- issue входит в coding stage из `Ready for Implementation`;
- `run <issue>` сам маршрутизирует issue в implementation flow;
- approved analysis artifacts используются как versioned вход;
- создается implementation branch/worktree;
- локальные проверки, commit, push, draft PR и CI оформлены как явный контракт;
- human review происходит после `Waiting for Code Review`, а не смешивается с
  analysis approval;
- merge implementation PR переводит issue в `Done` и завершает lifecycle без
  ручного post-merge разбора.

## Scope

В первую версию входит:

- отдельный SSOT `issue-implementation-flow`;
- stage-aware dispatch внутри `run`;
- implementation status model;
- stage-scoped runtime/session-binding;
- implementation launcher contract;
- finalization contract для commit/push/PR/status transitions;
- post-merge terminalization с отдельным review по GitHub-first reconcile;
- verification strategy для локальных тестов, CI и human review.

## Вне scope

- merge automation;
- deploy/release path;
- несколько implementation веток для одной issue;
- автоматическое принятие review;
- расширенный release/deploy flow после merge.

## Ограничения и предпосылки

- `run` остается единым issue-level entrypoint;
- analysis artifacts должны быть approved и versioned до запуска реализации;
- GitHub Project status остается source of truth по lifecycle issue;
- implementation runtime не должен перезаписывать analysis binding;
- repo-specific naming и launcher behavior должны оставаться configurable;
- проверки, которые могут задеть host `zellij`, выполняются только в
  headless-friendly среде.

## Follow-up acceptance 2026-03-15

Принятый
[ADR-0028](../../adr/0028-github-first-reconcile-and-runtime-cache-only.md)
зафиксировал, что:

- implementation flow не требует обязательного runtime-tracked PR identity;
- reconcile должен восстанавливаться из GitHub Project, канонического PR по
  branch contract и наблюдаемого git state.
