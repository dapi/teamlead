# ADR-0029: недоверенный контент GitHub как враждебный ввод

Статус: accepted
Дата: 2026-03-14

## Контекст

`ai-teamlead` запускается локально и использует GitHub issue, комментарии,
repo-local docs, project-local assets и runtime output как часть входного
контекста для агента.

Для public repos это создает принципиальную проблему: значительная часть
входного контента контролируется не владельцем локальной машины, а внешним
автором issue или любым другим участником репозитория.

Если считать этот контент "обычными инструкциями", то prompt injection и
execution abuse становятся не edge case, а базовой угрозой execution model.

## Решение

Контент из GitHub и публичного репозитория по умолчанию рассматривается как
hostile input, а не как часть trusted control plane.

Это правило распространяется на:

- issue body;
- comments;
- labels, titles и branch names;
- linked issues и linked PR;
- markdown-документы и project-local assets из public repo;
- shell output, test output и generated artifacts, если они возникли как
  следствие обработки недоверенного контента.

Следствия решения:

- недоверенный контент можно анализировать, но он не может напрямую задавать
  permission policy;
- текстовые инструкции внутри hostile input не считаются operator intent;
- repo-local assets из public repo не могут расширять trust scope сами по себе;
- runtime и documentation layer должны явно различать `operator instruction` и
  `content suggestion`.

## Последствия

Плюсы:

- появляется единая модель для prompt injection, execution abuse и data
  exfiltration risks;
- проще проектировать permission gates и safe mode как системный контракт;
- repo-local docs перестают быть неявным способом повысить привилегии.

Минусы:

- часть существующих assumptions о "полезных repo-local инструкциях" придется
  пересмотреть;
- понадобится явный trusted mechanism для assets, которым действительно нужно
  влиять на поведение runtime;
- увеличится число operator approvals в security-sensitive сценариях.

## Альтернативы

### 1. Считать hostile только issue и comments

Отклонено.

Файлы репозитория, generated artifacts и shell output тоже могут переносить
prompt injection или маскировать hostile content под внутренние инструкции.

### 2. Доверять repo-local assets, если репозиторий уже склонирован локально

Отклонено.

Факт локального clone не меняет происхождение и риск public content.

### 3. Решать проблему только operator guidance без runtime contract

Отклонено.

Без системного контракта policy быстро расползется между prompt-текстом,
привычками оператора и случайной реализацией.

## Связанные документы

- [../untrusted-input-security.md](../untrusted-input-security.md)
- [../features/0006-public-repo-security/README.md](../features/0006-public-repo-security/README.md)
- [./0030-public-repo-safe-mode-and-permission-gates.md](./0030-public-repo-safe-mode-and-permission-gates.md)

## Журнал изменений

### 2026-03-14

- создан ADR о трактовке GitHub и public-repo контента как hostile input
