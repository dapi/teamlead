# ADR-0011: Использовать release из `dapi/zellij-main` в CI

Статус: accepted
Дата: 2026-03-13

## Контекст

Для MVP нужен воспроизводимый способ тестировать `zellij`-launcher в CI.

Требования:

- тесты должны работать headless в Docker
- версия `zellij` должна быть pinned, а не floating
- CI не должен зависеть от системного `zellij` на runner
- локальная и CI-схемы должны использовать один и тот же CLI-контракт

Рассматривались варианты:

- использовать системный `zellij` из `apt`
- скачивать stable release `zellij-org/zellij`
- скачивать release из `dapi/zellij-main`

## Решение

Для docker-based integration tests используется release из
`dapi/zellij-main`.

Правила:

- pin хранится в файле `ZELLIJ_VERSION`
- файл содержит `tag` и `sha256`
- в CI бинарь скачивается из GitHub Releases `dapi/zellij-main`
- в контейнере он устанавливается сразу как `/usr/local/bin/zellij`
- launcher и тесты внутри контейнера работают с обычной командой `zellij`

## Последствия

Плюсы:

- воспроизводимый pinned runtime для launcher-тестов
- CI не зависит от версии `zellij` на GitHub runner
- код launcher не знает о `zellij-main` как об отдельной команде
- схема совпадает с уже проверенным подходом в соседних проектах

Минусы:

- появляется отдельный pin-файл `ZELLIJ_VERSION`
- нужен docker image build для integration job

## Связанные документы

- [README.md](../../README.md)
- [docs/features/0001-ai-teamlead-cli/02-how-we-build.md](../features/0001-ai-teamlead-cli/02-how-we-build.md)
- [docs/features/0001-ai-teamlead-cli/03-how-we-verify.md](../features/0001-ai-teamlead-cli/03-how-we-verify.md)

## Журнал изменений

### 2026-03-13

- создан ADR
