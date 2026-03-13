# Issue 3: Что строим

Статус: draft
Последнее обновление: 2026-03-13

## Problem

Проект уже содержит инфраструктурный слой, `run`/`poll`, project-local flow и
integration tests, но еще не подтверждено, что весь analysis workflow работает
достаточно хорошо на реальной issue этого репозитория, а не только на
тестовых doubles.

Без первого живого прогона команда не знает:

- где flow оказывается неудобным для оператора
- какие orchestration gaps не были видны в unit/integration test среде
- достаточно ли текущего SDD-контракта для реального analysis run

## Who Is It For

Основной пользователь этой задачи:

- владелец репозитория, который запускает `ai-teamlead` вручную и оценивает
  пригодность workflow для повседневной работы

Дополнительно результат нужен:

- будущему implementation flow, который будет опираться на versioned analysis-
  артефакты
- самому проекту `ai-teamlead`, который использует dogfooding как способ
  обнаружить реальные пробелы в UX и orchestration

## Scope

В рамках этой issue нужно:

- выбрать реальную issue этого репозитория для dogfooding run
- запустить `ai-teamlead run <issue-url>`
- пройти analysis workflow до `Waiting for Clarification` или
  `Waiting for Plan Review`
- сохранить versioned analysis-артефакты в `specs/issues/3/`
- зафиксировать реальные UX/orchestration gaps, проявившиеся на живом запуске
- явно отделить локальные находки текущего прогона от follow-up системных
  проблем, которые нужно вынести в отдельные issues

## Non-Goals

Вне scope этой issue:

- реализация follow-up исправлений, найденных по итогам прогона
- переделка `issue-analysis-flow` без подтвержденного gap
- автоматический переход к implementation flow
- полный production-grade e2e поверх всех внешних зависимостей на каждый запуск

## Motivation

Проекту нужен не еще один слой документации или тестовых заглушек, а проверка
того, как текущий workflow ведет себя в реальной операторской сессии на
собственной issue. Эта задача должна дать grounded feedback, а не теоретическую
уверенность.

## Operational Goal

Операционная цель задачи:

- доказать, что существующий manual run path можно использовать для настоящего
  анализа issue
- либо быстро показать, где именно flow блокируется или деградирует в реальном
  окружении

После выполнения задачи у владельца репозитория должно быть понятное основание
для одного из следующих действий:

- принять текущий flow как достаточно работоспособный для дальнейшего
  dogfooding
- или завести отдельные issues на blocking gaps с опорой на фактические
  симптомы прогона

## Constraints

- запуск должен использовать реальную issue и реальный `run` path, а не
  synthetic fixture
- результат должен быть оформлен как минимальный versioned SDD-комплект
- flow не должен перепрыгивать human gate и waiting-статусы
- итог должен быть полезен и человеку, и будущему агенту, который будет
  продолжать работу по артефактам

## Dependencies

- доступный `gh` и корректный доступ к репозиторию `dapi/ai-teamlead`
- настроенный GitHub Project из `./.ai-teamlead/settings.yml`
- рабочий launcher contract в `./.ai-teamlead/launch-agent.sh`
- доступный runtime-контекст для `zellij`, worktree и issue/session binding
- существующий системный SSOT `docs/issue-analysis-flow.md`
