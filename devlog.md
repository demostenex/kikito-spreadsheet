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

---

## 2026-05-01 — Milestone 4 concluído: UI funcional + publicação no GitHub

**Milestone:** M4
**Status:** Concluído

### O que foi feito
- `src/ui.rs` implementado com ratatui:
  - Header com nome do arquivo e tabs de sheets (sheet ativa destacada)
  - Tabela com scroll virtual (só renderiza linhas visíveis)
  - Largura de coluna calculada automaticamente por amostra de 50 linhas
  - Highlight de linha: `CURSOR_ROW_BG` (azul escuro)
  - Highlight de célula ativa: `ACTIVE_CELL_BG` (amarelo ouro, texto escuro) — contraste total com a linha
  - Highlight de resultados de busca: amarelo sobre linha
  - Status bar: `Ln x/y | Col x/y | NomeColuna › valor` — referência exata da célula atual
  - Help overlay com todos os atalhos (tecla `?`)
- `src/main.rs` com loop principal crossterm (raw mode, alternate screen)
- Testado com `DADOS - TELEFONE.xlsx`: 2519 linhas × 14 colunas, acentos, datas, CPFs
- Publicado no GitHub: https://github.com/demostenex/kikito-spreadsheet
- Arquivo `tests/fixtures/telefone.xlsx` bloqueado no `.gitignore` — nenhum dado pessoal subiu

### Decisões tomadas
- `CURSOR_ROW_BG` e `ACTIVE_CELL_BG` separados: a linha e a célula precisam de cores distintas para o highlight de célula ser visível
- Row style do ratatui sobrescreve cell style se forem a mesma cor — solução: cores diferentes nas duas constantes
- Borrow checker: `sync_offsets` (mutável) deve ser chamado antes de `current_sheet()` (imutável) na função `draw_table`
- Fixtures pessoais ignorados via `.gitignore` glob: `tests/fixtures/*.xlsx`

### Problemas encontrados
- `assert_fs` puxava `globset 0.4.18` que exige edition2024 (Rust < 1.85 não suporta) — removido, `tempfile` suficiente
- `DataType` no calamine 0.26 é trait; enum correto é `Data` — corrigido
- `ExcelDateTime` não é `f64`; usar `.as_f64()` — corrigido
- `open_workbook` precisa de tipo explícito no `map_err` (ex: `XlsxError`) — corrigido
- Rust atualizado de 1.79 → 1.95 para suportar dependências transitivas com edition2024

### Próxima sessão (Milestone 5 — Edição estilo Vim)
- Adicionar `rust_xlsxwriter` ao Cargo.toml
- Implementar pending key buffer para sequências (`gg`, `dd`, `yy`)
- Modo Insert: buffer de edição com cursor visual na célula
- Modo Command: `:w`, `:q`, `:wq`, `:q!`
- Operações: `o/O` (nova linha), `dd` (deletar), `yy/p/P` (copiar/colar), `x` (limpar), `u/Ctrl+R` (undo/redo)
- Escritores: `CsvWriter`, `XlsxWriter`
- Indicador `[+]` no header quando arquivo modificado

---

## 2026-05-01 — M5 edição vim + M6 README + fixes

**Milestone:** M5 + M6
**Status:** Concluído

### O que foi feito
- Edição estilo vim completa: Insert mode, Command mode, undo/redo, dd/yy/p/x/o/O
- Writers: CsvWriter e XlsxWriter (rust_xlsxwriter)
- `:número` navega para linha (1-indexado igual ao contador Ln x/y da status bar)
- Fix: `n`/`N` em modo busca eram tratados como navegação em vez de caracteres literais — corrigido para adicionar ao buffer de query; em Normal mode continuam navegando entre resultados
- Busca reutilizável: após Esc os resultados e highlights permanecem, `n`/`N` continuam ciclando
- README.md publicado com ASCII demo (dados fictícios), instalação, atalhos, formatos e stack
- plan.md e devlog.md atualizados com todos os milestones

### Decisões tomadas
- CPFs e dados reais removidos do README antes de publicar — dados de demonstração são sempre fictícios
- `:número` usa a mesma numeração do contador `Ln x/y` da status bar para não confundir o usuário
- n/N em search mode → literal char; n/N em normal mode → navegação entre hits (comportamento vim)

### Resultado final
```
running 80 tests
test result: ok. 80 passed; 0 failed; 0 ignored
```

Repositório: https://github.com/demostenex/kikito-spreadsheet
