# Issue 8: release flow, установка через `brew`/`curl`, версионирование и changelog

Статус: draft
Тип задачи: `feature`
Тип проекта: `infra/platform`
Размер: `medium`
Последнее обновление: 2026-03-15
Статус согласования: pending human review

## Issue

- GitHub issue: https://github.com/dapi/ai-teamlead/issues/8
- GitHub issue title: `Релизный флоу: CI-релизы, установка через brew/curl, версионирование, changelog`

## Summary

Сейчас репозиторий умеет собирать и проверять `ai-teamlead` только как
внутренний development-артефакт:

- в `.github/workflows/ci.yml` есть проверки `fmt`, `check`, `test` и
  docker-based integration tests;
- version уже есть в `Cargo.toml`, но она не связана с публичным release flow;
- нет канонического tag-driven процесса публикации бинарей;
- нет user-facing install path через `brew` и `curl`;
- changelog не оформлен как обязательный вход в release.

Issue вводит минимальный публичный release contract для Rust CLI:

- один operator-facing entrypoint релиза;
- semver-тег `vX.Y.Z` как trigger релиза;
- простой operator-facing path для инкремента `major` / `minor` / `patch`;
- один source of truth для версии;
- полное соответствие Semantic Versioning 2.0.0;
- GitHub Release с reproducible артефактами и checksum-файлами;
- install path через `brew` и `curl`, опирающийся на те же release assets;
- changelog как обязательная часть release-пакета;
- отдельные Release Notes как самостоятельный артефакт, отличный от
  `CHANGELOG.md`;
- release automation в CI без ручной сборки бинарей.

## Status

Analysis-пакет подготовлен и готов к human review.

## Artifacts

- [01-what-we-build.md](./01-what-we-build.md)
- [02-how-we-build.md](./02-how-we-build.md)
- [03-how-we-verify.md](./03-how-we-verify.md)
- [04-implementation-plan.md](./04-implementation-plan.md)

## Related Context

- [../../../README.md](../../../README.md)
- [../../../ROADMAP.md](../../../ROADMAP.md)
- [../../../docs/code-quality.md](../../../docs/code-quality.md)
- [../../../docs/untrusted-input-security.md](../../../docs/untrusted-input-security.md)
- [../../../docs/issue-analysis-flow.md](../../../docs/issue-analysis-flow.md)
- [../../../docs/documentation-process.md](../../../docs/documentation-process.md)
- [../../../docs/implementation-plan.md](../../../docs/implementation-plan.md)
- [../../../docs/features/0001-ai-teamlead-cli/README.md](../../../docs/features/0001-ai-teamlead-cli/README.md)
- [../../../docs/features/0002-repo-init/README.md](../../../docs/features/0002-repo-init/README.md)
- [../../../docs/features/0004-issue-implementation-flow/README.md](../../../docs/features/0004-issue-implementation-flow/README.md)
- [../../../docs/features/0006-public-repo-security/README.md](../../../docs/features/0006-public-repo-security/README.md)
- [../../../docs/adr/0011-use-zellij-main-release-in-ci.md](../../../docs/adr/0011-use-zellij-main-release-in-ci.md)
- [../51/README.md](../51/README.md)

## Open Questions

Блокирующих открытых вопросов по текущему scope не выявлено.

В analysis-пакете зафиксированы:

- штатный release path через `--bump <major|minor|patch>`;
- ограниченный exact-version path только для bootstrap/recovery;
- канонический release tooling первой версии: `cargo-dist`;
- канонический Homebrew tap первой версии `dapi/homebrew-ai-teamlead`;
- канонический tap update path: direct commit из CI;
- минимальный формат Release Notes и guide для их генерации;
- integrity contract для `curl`: latest stable только из GitHub Releases и
  обязательная checksum-проверка перед установкой;
- bootstrap-контракт первого публичного релиза без обязательного backfill
  прошлых версий;
- fail-closed поведение при partial publish и повторном запуске релиза.
