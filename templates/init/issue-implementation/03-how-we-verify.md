# Stage 3: Проверить и завершить

- выполни релевантные unit, integration и smoke-friendly проверки
- сверь результат с approved verification criteria
- если локальные проверки пройдены и PR готов, заверши stage через
  `complete-stage --stage implementation --outcome ready-for-ci`
- если stage заблокирован, используй `--outcome blocked`
