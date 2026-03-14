#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: review-docs.sh [options] [path...]

Запускает параллельное ревью документации по аспектам:
- gaps
- completeness
- contradictions
- consistency

Options:
  --aspects LIST     Comma-separated list of aspects. Default:
                     gaps,completeness,contradictions,consistency
  --out-dir DIR      Directory for generated reports.
                     Default: .docvalidate/reviews/<timestamp>
  --model MODEL      Pass model to codex exec.
  --reasoning LEVEL  Codex reasoning effort: low, medium, high.
                     Default: medium
  --timeout-seconds N
                     Kill a single codex exec after N seconds.
                     Default: 300
  --no-synthesis     Skip final summary agent.
  -h, --help         Show help.

If no paths are provided, the command reviews:
- README.md
- docs

Ignored review paths:
- .ai-teamlead
EOF
}

die() {
    printf 'review-docs.sh: %s\n' "$1" >&2
    exit 1
}

trim() {
    local value="$1"
    value="${value#"${value%%[![:space:]]*}"}"
    value="${value%"${value##*[![:space:]]}"}"
    printf '%s' "$value"
}

normalize_repo_relative_path() {
    local path="$1"
    path="${path#./}"
    path="${path%/}"
    if [[ -z "$path" ]]; then
        path="."
    fi
    printf '%s' "$path"
}

is_ignored_path() {
    local path=""
    local ignored=""

    path="$(normalize_repo_relative_path "$1")"
    for ignored in "${IGNORED_PATHS[@]}"; do
        if [[ "$path" == "$ignored" || "$path" == "$ignored/"* ]]; then
            return 0
        fi
    done

    return 1
}

expand_target_list() {
    local target=""
    local -a expanded=()
    local normalized_target=""

    for target in "$@"; do
        normalized_target="$(normalize_repo_relative_path "$target")"

        if is_ignored_path "$normalized_target"; then
            printf 'review-docs.sh: ignoring path from review scope: %s\n' "$target" >&2
            continue
        fi

        if [[ -d "$target" ]]; then
            while IFS= read -r file; do
                if is_ignored_path "$file"; then
                    continue
                fi
                expanded+=("$file")
            done < <(
                find "$target" \
                    \( -path '*/.ai-teamlead' -o -path '*/.ai-teamlead/*' \) -prune \
                    -o -type f -print | sort
            )
        else
            expanded+=("$target")
        fi
    done

    printf '%s\n' "${expanded[@]}"
}

join_lines() {
    local first=1
    local item=""

    for item in "$@"; do
        if [[ $first -eq 1 ]]; then
            printf '%s' "$item"
            first=0
        else
            printf '\n%s' "$item"
        fi
    done
}

build_common_prompt() {
    local aspect="$1"
    local targets_block governance_block focus_text

    targets_block="$(join_lines "${TARGETS[@]}")"
    governance_block="$(join_lines "${GOVERNANCE_PATHS[@]}")"

    case "$aspect" in
        gaps)
            focus_text="Ищи смысловые и структурные пробелы: отсутствующие разделы, недоописанные контракты, пропущенные зависимости, неописанные ограничения и missing links между документами."
            ;;
        completeness)
            focus_text="Проверяй полноту относительно repo-level правил. Смотри, покрыты ли оси 'Что строим', 'Как строим', 'Как проверяем', хватает ли критериев готовности, инвариантов, сценариев проверки и явного scope."
            ;;
        contradictions)
            focus_text="Ищи противоречия между документами: несовместимые статусы, разные naming contracts, конфликтующие требования, различающиеся promises about behavior, conflicting source-of-truth claims."
            ;;
        consistency)
            focus_text="Проверяй консистентность формы и терминологии: одинаковые имена статусов, единый стиль ссылок, согласованность терминов feature/flow/ADR, одинаковое использование путей, конфиг-ключей и runtime-понятий."
            ;;
        *)
            die "unsupported aspect: $aspect"
            ;;
    esac

    cat <<EOF
Ты делаешь reviewer-only аудит документации репозитория.
Ничего не меняй и не предлагай патчи; нужен только список findings.
Это дочерний одиночный reviewer-run внутри внешнего launcher-а.
Не используй repo-local skills, не запускай ./scripts/review-docs.sh, не
запускай параллельные подагенты и не пытайся оркестрировать дополнительные
review-сессии.

Фокус ревью: $aspect
$focus_text

Сначала прочитай governance-документы, если они существуют:
$governance_block

Потом проверь target paths:
$targets_block

Требования к ответу:
1. Отвечай на русском языке.
2. Дай только findings по делу, без длинного пересказа.
3. У каждого finding должны быть:
   - Severity: high, medium или low
   - Короткий заголовок
   - Почему это проблема
   - Evidence с точными file references
   - Что нужно уточнить или дописать
4. Если явных проблем нет, напиши: "Findings: none".
5. Не дублируй одно и то же наблюдение в нескольких формулировках.
EOF
}

build_summary_prompt() {
    local reports_block
    reports_block="$(join_lines "${REPORT_FILES[@]}")"

    cat <<EOF
Ты собираешь итоговый отчет по нескольким параллельным ревью документации.
Это дочерний synthesis-run внутри внешнего launcher-а.
Не используй repo-local skills, не запускай ./scripts/review-docs.sh и не
запускай дополнительные подагенты.

Прочитай aspect-отчеты:
$reports_block

Собери единый результат на русском языке в формате:

# Documentation Review Summary

## Главные findings
- ...

## Противоречия и дубли
- ...

## Что чинить сначала
1. ...
2. ...
3. ...

Правила:
1. Дедуплицируй одинаковые findings из разных аспектов.
2. Если несколько отчетов ссылаются на одну проблему, объедини это в один
   finding и укажи все релевантные file references.
3. Если все aspect-отчеты пустые, явно напиши, что критичных проблем не найдено.
4. Не предлагай редактирование файлов в diff-формате.
EOF
}

run_aspect() {
    local aspect="$1"
    local output_file="$OUT_DIR/$aspect.md"
    local log_file="$OUT_DIR/$aspect.log"
    local prompt
    local -a cmd

    prompt="$(build_common_prompt "$aspect")"

    cmd=(
        timeout
        "${TIMEOUT_SECONDS}s"
        codex
        -a never
        exec
        --skip-git-repo-check
        -C "$EXEC_ROOT"
        --add-dir "$REPO_ROOT"
        -c "model_reasoning_effort=\"$REASONING_EFFORT\""
        -s read-only
        --color never
        -o "$output_file"
    )
    if [[ -n "$MODEL" ]]; then
        cmd+=(-m "$MODEL")
    fi
    cmd+=("$prompt")

    "${cmd[@]}" >"$log_file" 2>&1 &
    PIDS+=("$!")
    PID_ASPECTS+=("$aspect")
    REPORT_FILES+=("$output_file")
}

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[[ -n "$REPO_ROOT" ]] || die "must be run inside a git repository"

cd "$REPO_ROOT"

command -v codex >/dev/null 2>&1 || die "codex is not available in PATH"

EXEC_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/doc-review-codex-XXXXXX")"
cleanup() {
    rm -rf "$EXEC_ROOT"
}
trap cleanup EXIT

declare -a TARGETS=()
declare -a GOVERNANCE_PATHS=()
declare -a REPORT_FILES=()
declare -a PIDS=()
declare -a PID_ASPECTS=()
declare -a IGNORED_PATHS=(".ai-teamlead")

declare -a DEFAULT_TARGETS=("README.md" "docs")
declare -a DEFAULT_ASPECTS=("gaps" "completeness" "contradictions" "consistency")

MODEL=""
OUT_DIR=""
NO_SYNTHESIS=0
REASONING_EFFORT="${DOC_REVIEW_REASONING_EFFORT:-medium}"
TIMEOUT_SECONDS="${DOC_REVIEW_TIMEOUT_SECONDS:-300}"
declare -a ASPECTS=("${DEFAULT_ASPECTS[@]}")

while [[ $# -gt 0 ]]; do
    case "$1" in
        --aspects)
            [[ $# -ge 2 ]] || die "--aspects requires a value"
            IFS=',' read -r -a ASPECTS <<<"$2"
            shift 2
            ;;
        --out-dir)
            [[ $# -ge 2 ]] || die "--out-dir requires a value"
            OUT_DIR="$2"
            shift 2
            ;;
        --model)
            [[ $# -ge 2 ]] || die "--model requires a value"
            MODEL="$2"
            shift 2
            ;;
        --reasoning)
            [[ $# -ge 2 ]] || die "--reasoning requires a value"
            REASONING_EFFORT="$2"
            shift 2
            ;;
        --timeout-seconds)
            [[ $# -ge 2 ]] || die "--timeout-seconds requires a value"
            TIMEOUT_SECONDS="$2"
            shift 2
            ;;
        --no-synthesis)
            NO_SYNTHESIS=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            while [[ $# -gt 0 ]]; do
                TARGETS+=("$1")
                shift
            done
            ;;
        -*)
            die "unknown option: $1"
            ;;
        *)
            TARGETS+=("$1")
            shift
            ;;
    esac
done

if [[ ${#TARGETS[@]} -eq 0 ]]; then
    for target in "${DEFAULT_TARGETS[@]}"; do
        if [[ -e "$target" ]]; then
            TARGETS+=("$target")
        fi
    done
fi

[[ ${#TARGETS[@]} -gt 0 ]] || die "no documentation targets found"

for target in "${TARGETS[@]}"; do
    [[ -e "$target" ]] || die "target path does not exist: $target"
done

for idx in "${!ASPECTS[@]}"; do
    ASPECTS[$idx]="$(trim "${ASPECTS[$idx]}")"
    [[ -n "${ASPECTS[$idx]}" ]] || die "empty aspect in --aspects"
done

case "$REASONING_EFFORT" in
    low|medium|high) ;;
    *)
        die "--reasoning must be one of: low, medium, high"
        ;;
esac

[[ "$TIMEOUT_SECONDS" =~ ^[0-9]+$ ]] || die "--timeout-seconds must be a positive integer"
[[ "$TIMEOUT_SECONDS" -ge 1 ]] || die "--timeout-seconds must be >= 1"

for governance_path in \
    "AGENTS.md" \
    "README.md" \
    "docs/documentation-structure.md" \
    "docs/code-quality.md" \
    "docs/issue-analysis-flow.md"
do
    if [[ -e "$governance_path" ]]; then
        GOVERNANCE_PATHS+=("$governance_path")
    fi
done

[[ ${#GOVERNANCE_PATHS[@]} -gt 0 ]] || die "no governance documentation found"

if [[ -z "$OUT_DIR" ]]; then
    OUT_DIR="$REPO_ROOT/.docvalidate/reviews/$(date +%Y%m%d-%H%M%S)"
elif [[ "$OUT_DIR" != /* ]]; then
    OUT_DIR="$REPO_ROOT/$OUT_DIR"
fi

mapfile -t TARGETS < <(expand_target_list "${TARGETS[@]}")
[[ ${#TARGETS[@]} -gt 0 ]] || die "expanded target list is empty"

for idx in "${!TARGETS[@]}"; do
    TARGETS[$idx]="$(realpath -m "$REPO_ROOT/${TARGETS[$idx]}")"
done

for idx in "${!GOVERNANCE_PATHS[@]}"; do
    GOVERNANCE_PATHS[$idx]="$(realpath -m "$REPO_ROOT/${GOVERNANCE_PATHS[$idx]}")"
done

mkdir -p "$OUT_DIR"

cat >"$OUT_DIR/run-context.txt" <<EOF
repo_root=$REPO_ROOT
targets=$(printf '%s;' "${TARGETS[@]}")
aspects=$(printf '%s;' "${ASPECTS[@]}")
model=${MODEL:-default}
reasoning=$REASONING_EFFORT
timeout_seconds=$TIMEOUT_SECONDS
EOF

for aspect in "${ASPECTS[@]}"; do
    run_aspect "$aspect"
done

FAILED=0

for idx in "${!PIDS[@]}"; do
    if ! wait "${PIDS[$idx]}"; then
        printf 'review-docs.sh: aspect failed: %s\n' "${PID_ASPECTS[$idx]}" >&2
        FAILED=1
    fi
done

if [[ "$FAILED" -ne 0 ]]; then
    exit 1
fi

if [[ "$NO_SYNTHESIS" -eq 0 ]]; then
    SUMMARY_FILE="$OUT_DIR/summary.md"
    SUMMARY_LOG="$OUT_DIR/summary.log"
    SUMMARY_PROMPT="$(build_summary_prompt)"
    declare -a SUMMARY_CMD

    SUMMARY_CMD=(
        timeout
        "${TIMEOUT_SECONDS}s"
        codex
        -a never
        exec
        --skip-git-repo-check
        -C "$EXEC_ROOT"
        --add-dir "$REPO_ROOT"
        -c "model_reasoning_effort=\"$REASONING_EFFORT\""
        -s read-only
        --color never
        -o "$SUMMARY_FILE"
    )
    if [[ -n "$MODEL" ]]; then
        SUMMARY_CMD+=(-m "$MODEL")
    fi
    SUMMARY_CMD+=("$SUMMARY_PROMPT")

    "${SUMMARY_CMD[@]}" >"$SUMMARY_LOG" 2>&1
fi

printf 'Documentation review artifacts: %s\n' "$OUT_DIR"
if [[ "$NO_SYNTHESIS" -eq 0 ]]; then
    printf 'Summary: %s\n' "$OUT_DIR/summary.md"
fi
