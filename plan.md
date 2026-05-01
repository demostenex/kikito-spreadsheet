# excel-tui — Plano de Desenvolvimento (TDD)

> TUI em Rust para leitura de arquivos Excel (xls, xlsx) e CSV no terminal.

---

## Stack Tecnológica

| Camada       | Crate           | Função                              |
|--------------|-----------------|-------------------------------------|
| TUI          | `ratatui`       | Renderização de widgets no terminal |
| Backend      | `crossterm`     | Input de teclado / mouse / raw mode |
| Excel        | `calamine`      | Leitura de xls / xlsx               |
| CSV          | `csv`           | Leitura de CSV com detecção de delimitador |
| Erros        | `thiserror`     | Tipos de erro ergonômicos           |
| Testes       | built-in + `assert_fs` | Fixtures de arquivos temporários |

---

## Estrutura de Diretórios

```
excel-tui/
├── src/
│   ├── main.rs          — entrypoint, args, loop principal
│   ├── app.rs           — estado da aplicação (App struct)
│   ├── event.rs         — eventos de teclado/mouse
│   ├── ui.rs            — composição dos widgets ratatui
│   ├── reader/
│   │   ├── mod.rs       — trait Reader + factory
│   │   ├── csv.rs       — leitor CSV
│   │   ├── xlsx.rs      — leitor XLSX
│   │   └── xls.rs       — leitor XLS legado
│   └── table.rs         — estrutura de dados TableData / Sheet
├── tests/
│   ├── fixtures/        — arquivos .csv, .xlsx, .xls de teste
│   ├── reader_tests.rs
│   └── app_tests.rs
├── Cargo.toml
├── plan.md              — este arquivo
└── devlog.md            — diário de bordo
```

---

## Fases e Milestones

### Milestone 1 — Setup + Modelos de Dados
**Meta:** projeto compilando com estruturas core testadas.

#### Tarefas
- [ ] `cargo new excel-tui` com Cargo.toml inicial
- [ ] Adicionar todas as dependências
- [ ] Criar `TableData`, `Sheet`, `Cell` em `table.rs`
- [ ] Criar trait `Reader` em `reader/mod.rs`

#### Testes (escrever antes do código)
```
table::tests::cell_display_string
table::tests::cell_display_number
table::tests::cell_display_empty
table::tests::sheet_row_count
table::tests::sheet_col_count
table::tests::tabledata_first_sheet
table::tests::tabledata_sheet_by_name
```

---

### Milestone 2 — Leitores de Arquivo
**Meta:** CSV, XLSX e XLS lidos e convertidos para `TableData`.

#### Tarefas
- [ ] Implementar `CsvReader` em `reader/csv.rs`
- [ ] Implementar `XlsxReader` em `reader/xlsx.rs`
- [ ] Implementar `XlsReader` em `reader/xls.rs`
- [ ] Factory `reader_for(path) -> Box<dyn Reader>`
- [ ] Criar fixtures de teste: `tests/fixtures/`
  - `sample.csv`, `sample.xlsx`, `sample.xls`
  - `empty.csv`, `single_col.csv`
  - `multisheet.xlsx`

#### Testes
```
reader::csv::tests::reads_simple_csv
reader::csv::tests::reads_semicolon_delimited
reader::csv::tests::reads_empty_csv
reader::csv::tests::reads_utf8_with_bom
reader::xlsx::tests::reads_single_sheet
reader::xlsx::tests::reads_multiple_sheets
reader::xlsx::tests::reads_numbers_and_strings
reader::xls::tests::reads_legacy_xls
reader::mod::tests::factory_csv
reader::mod::tests::factory_xlsx
reader::mod::tests::factory_xls
reader::mod::tests::factory_unknown_extension_error
```

---

### Milestone 3 — Estado da Aplicação (App)
**Meta:** máquina de estados navegável e testável sem UI.

#### Tarefas
- [ ] `App` struct com campos: `data`, `cursor`, `scroll`, `active_sheet`, `mode`
- [ ] Métodos: `scroll_down`, `scroll_up`, `scroll_left`, `scroll_right`
- [ ] Métodos: `next_sheet`, `prev_sheet`
- [ ] Enum `Mode`: `Normal`, `Search`, `Help`
- [ ] Busca: `enter_search`, `update_search`, `exit_search`

#### Testes
```
app::tests::initial_cursor_is_zero
app::tests::scroll_down_moves_cursor
app::tests::scroll_down_clamps_at_last_row
app::tests::scroll_up_clamps_at_zero
app::tests::scroll_right_clamps_at_last_col
app::tests::next_sheet_wraps_around
app::tests::prev_sheet_wraps_around
app::tests::search_filters_rows
app::tests::search_empty_shows_all
```

---

### Milestone 4 — Renderização TUI
**Meta:** tela funcional no terminal com table, status bar e help overlay.

#### Tarefas
- [ ] Layout principal: header + table + status bar
- [ ] Widget de tabela com scroll virtual (apenas linhas visíveis)
- [ ] Cabeçalho com nome do arquivo e sheet ativa
- [ ] Status bar: linha/coluna atual, total de linhas
- [ ] Help overlay (tecla `?`)
- [ ] Highlight da linha selecionada

#### Testes (integração visual)
```
ui::tests::renders_without_panic_empty_data
ui::tests::renders_without_panic_large_data
ui::tests::status_bar_shows_correct_position
```

---

### Milestone 5 — Busca e Navegação Avançada
**Meta:** busca inline, atalhos de teclado completos.

#### Tarefas
- [ ] Modo busca ativado com `/`
- [ ] Highlight das células que fazem match
- [ ] Navegação entre resultados com `n` / `N`
- [ ] Ir para primeira / última linha: `g` / `G`
- [ ] Ir para primeira / última coluna: `0` / `$`
- [ ] Redimensionar colunas: `+` / `-`

#### Testes
```
app::tests::search_next_cycles_results
app::tests::search_prev_cycles_results
app::tests::goto_last_row
app::tests::goto_first_col
```

---

### Milestone 6 — Polimento e Entrega
**Meta:** binário publicável, README completo no GitHub.

#### Tarefas
- [ ] Argumentos CLI: `excel-tui arquivo.xlsx`
- [ ] Mensagem de erro amigável para arquivo inválido
- [ ] Ícone / ASCII art no splash
- [ ] README.md completo (instalação, uso, screenshots, atalhos)
- [ ] `cargo clippy` sem warnings
- [ ] `cargo test` 100% passando
- [ ] Release binary via `cargo build --release`

---

## Atalhos de Teclado (Design)

| Tecla         | Ação                          |
|---------------|-------------------------------|
| `j` / `↓`    | Linha abaixo                  |
| `k` / `↑`    | Linha acima                   |
| `h` / `←`    | Coluna à esquerda             |
| `l` / `→`    | Coluna à direita              |
| `g`           | Primeira linha                |
| `G`           | Última linha                  |
| `0`           | Primeira coluna               |
| `$`           | Última coluna                 |
| `Tab`         | Próxima sheet                 |
| `Shift+Tab`   | Sheet anterior                |
| `/`           | Entrar no modo busca          |
| `n` / `N`    | Próximo / anterior resultado  |
| `Esc`         | Sair do modo busca            |
| `?`           | Toggle help overlay           |
| `q`           | Sair                          |

---

## Critérios de Aceite por Feature

### Leitor CSV
- Detecta automaticamente delimitador (`,` `;` `\t`)
- Suporta aspas e campos com quebra de linha
- Suporta UTF-8 com e sem BOM

### Leitor XLSX/XLS
- Lê todas as sheets disponíveis
- Preserva tipos: texto, número, data, booleano
- Não falha em células vazias ou mergeadas

### TUI
- Funciona em terminais 80x24 e maiores
- Não quebra com dados Unicode (emoji, acentos, CJK)
- Scroll suave em arquivos com 100k+ linhas

---

## Convenções de Código

- Sem `unwrap()` fora de testes
- Erros propagados com `?` e tipo `AppError` (thiserror)
- Módulos com `#[cfg(test)]` inline para unit tests
- Integração em `tests/` com fixtures reais
- Sem comentários óbvios; apenas invariantes não-triviais
