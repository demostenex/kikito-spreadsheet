# Diário de Bordo — excel-tui

> Registro diário do desenvolvimento. Cada entrada descreve o que foi planejado, o que foi feito, decisões tomadas e próximos passos.

---

## Formato de Entrada

```
## YYYY-MM-DD — Título da sessão

**Milestone:** M#
**Status:** Em andamento | Concluído | Bloqueado

### O que foi feito
- ...

### Decisões tomadas
- ...

### Problemas encontrados
- ...

### Próxima sessão
- ...
```

---

## 2026-05-01 — Planejamento inicial e estrutura do projeto

**Milestone:** M1
**Status:** Em andamento

### O que foi feito
- Definição da stack tecnológica: Rust + ratatui + calamine + csv
- Criação do `plan.md` com todas as fases, testes planejados e critérios de aceite
- Criação deste diário de bordo (`devlog.md`)
- Decisão de usar TDD: testes escritos antes da implementação em cada milestone

### Decisões tomadas
- **Rust sobre Python:** escolha consciente por performance e binário nativo distribuível,
  mesmo que o tempo de desenvolvimento seja maior.
- **ratatui sobre outras libs:** é o sucessor ativo do `tui-rs`, bem mantido e com boa
  documentação. `cursive` e `egui` foram descartados.
- **calamine para Excel:** suporta xls e xlsx no mesmo crate sem dependências nativas
  (libxlsxwriter, etc.), facilitando cross-compile futuro.
- **thiserror para erros:** mais ergonômico que `anyhow` quando os tipos de erro precisam
  ser distintos e testáveis.
- **Atalhos estilo vim (hjkl):** familiar para usuários de terminal e consistente com
  ferramentas como `k9s`, `lazygit`, etc.

### Próxima sessão
- Executar `cargo new excel-tui` e configurar `Cargo.toml` com todas as dependências
- Criar `table.rs` com `Cell`, `Sheet`, `TableData`
- Escrever os unit tests de M1 antes de implementar (TDD)
- Meta: M1 concluído com `cargo test` verde

---

---

## 2026-05-01 — Milestone 1 concluído: modelos, leitores e estado da app

**Milestone:** M1 + M2 + M3 (parcial)
**Status:** Concluído

### O que foi feito
- `cargo init` e `Cargo.toml` com todas as dependências
- `src/table.rs`: `Cell`, `Sheet`, `TableData` com 16 testes unitários
- `src/reader/mod.rs`: trait `Reader` + factory `reader_for()` com 5 testes
- `src/reader/csv.rs`: `CsvReader` com detecção de delimitador (`,` `;` `\t`), suporte a BOM UTF-8 — 7 testes
- `src/reader/xlsx.rs`: `XlsxReader` via calamine — 3 testes com fixtures reais
- `src/reader/xls.rs`: `XlsReader` via calamine — 1 teste com fixture real
- `src/app.rs`: `App` com scroll, navegação de sheets, busca — 14 testes
- `src/event.rs`: enum `AppEvent` + `poll_event` + mapeamento de teclas
- `src/error.rs`: `AppError` com thiserror
- `src/ui.rs`: stub para Milestone 4
- `tests/fixtures/`: `sample.xlsx`, `multisheet.xlsx`, `sample.xls` gerados via Python
- Rust atualizado de 1.79 → 1.95 (dependências transitivas exigiam edition2024)

### Decisões tomadas
- `DataType` no calamine 0.26 é um trait; o enum é `Data` — corrigido
- `ExcelDateTime` não é `f64` diretamente; usa-se `.as_f64()` — corrigido
- `map_err` em closures com `open_workbook` exige tipo explícito (ex: `XlsxError`) para inferência
- `assert_fs` removido (puxa globset 0.4.18 que exige edition2024); `tempfile` suficiente

### Resultado
```
running 49 tests
test result: ok. 49 passed; 0 failed; 0 ignored
```

### Próxima sessão (Milestone 4 — UI)
- Implementar `src/ui.rs` com ratatui: tabela, cabeçalho, status bar, help overlay
- Loop principal em `src/main.rs` com crossterm raw mode
- Meta: binário funcional e navegável no terminal

<!-- Entradas futuras serão adicionadas abaixo -->
