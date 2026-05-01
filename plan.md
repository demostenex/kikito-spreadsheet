# excel-tui — Plano de Desenvolvimento (TDD)

> TUI em Rust para leitura e **edição** de arquivos Excel (xls, xlsx) e CSV no terminal.

---

## Stack Tecnológica

| Camada       | Crate               | Função                                      |
|--------------|---------------------|---------------------------------------------|
| TUI          | `ratatui`           | Renderização de widgets no terminal         |
| Backend      | `crossterm`         | Input de teclado / mouse / raw mode         |
| Leitura      | `calamine`          | Leitura de xls / xlsx                       |
| Escrita      | `rust_xlsxwriter`   | Escrita de xlsx                             |
| CSV          | `csv`               | Leitura e escrita de CSV                    |
| Erros        | `thiserror`         | Tipos de erro ergonômicos                   |
| Testes       | built-in + `tempfile` | Fixtures de arquivos temporários          |

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
│   └── writer/
│       ├── mod.rs       — trait Writer + factory
│       ├── csv.rs       — escritor CSV
│       └── xlsx.rs      — escritor XLSX
├── tests/
│   ├── fixtures/        — arquivos de teste (ignorados pelo git)
│   ├── reader_tests.rs
│   └── writer_tests.rs
├── Cargo.toml
├── plan.md              — este arquivo
└── devlog.md            — diário de bordo
```

---

## Modos da Aplicação (estilo Vim)

| Modo      | Como entrar         | Como sair        | Descrição                        |
|-----------|---------------------|------------------|----------------------------------|
| `Normal`  | `Esc` / iniciar     | —                | Navegação e comandos             |
| `Insert`  | `i`, `o`, `O`       | `Esc`            | Edição de célula                 |
| `Command` | `:`                 | `Esc` / `Enter`  | Comandos como `:w`, `:q`         |
| `Search`  | `/`                 | `Esc`            | Busca inline                     |
| `Help`    | `?`                 | `?` ou `Esc`     | Overlay de ajuda                 |

---

## Atalhos de Teclado Completos

### Modo Normal — Navegação

| Tecla         | Ação                              |
|---------------|-----------------------------------|
| `j` / `↓`    | Linha abaixo                      |
| `k` / `↑`    | Linha acima                       |
| `h` / `←`    | Coluna à esquerda                 |
| `l` / `→`    | Coluna à direita                  |
| `gg`          | Primeira linha (sequência vim)    |
| `G`           | Última linha                      |
| `0`           | Primeira coluna                   |
| `$`           | Última coluna                     |
| `Tab`         | Próxima sheet                     |
| `Shift+Tab`   | Sheet anterior                    |

### Modo Normal — Edição

| Tecla         | Ação                              |
|---------------|-----------------------------------|
| `i`           | Editar célula atual (Insert)      |
| `o`           | Nova linha abaixo + Insert        |
| `O`           | Nova linha acima + Insert         |
| `dd`          | Deletar linha atual               |
| `yy`          | Copiar (yank) linha atual         |
| `p`           | Colar linha abaixo                |
| `P`           | Colar linha acima                 |
| `x`           | Limpar conteúdo da célula         |
| `u`           | Desfazer (undo)                   |
| `Ctrl+R`      | Refazer (redo)                    |

### Modo Normal — Geral

| Tecla         | Ação                              |
|---------------|-----------------------------------|
| `/`           | Entrar no modo busca              |
| `n` / `N`    | Próximo / anterior resultado      |
| `:`           | Entrar no modo comando            |
| `?`           | Toggle help overlay               |
| `q`           | Sair (pede confirmação se dirty)  |

### Modo Insert

| Tecla         | Ação                              |
|---------------|-----------------------------------|
| Qualquer char | Adiciona ao buffer de edição      |
| `Backspace`   | Apaga último caractere            |
| `Enter`       | Confirma edição                   |
| `Esc`         | Cancela edição                    |

### Modo Command

| Comando       | Ação                              |
|---------------|-----------------------------------|
| `:w`          | Salvar arquivo                    |
| `:q`          | Sair (bloqueia se modificado)     |
| `:wq`         | Salvar e sair                     |
| `:q!`         | Sair sem salvar (força)           |

### Modo Search

| Tecla         | Ação                              |
|---------------|-----------------------------------|
| Qualquer char | Refina a busca em tempo real      |
| `Backspace`   | Apaga último caractere da busca   |
| `n` / `N`    | Próximo / anterior resultado      |
| `Esc`         | Sair da busca                     |

---

## Fases e Milestones

### ✅ Milestone 1 — Setup + Modelos de Dados
**Concluído em:** 2026-05-01

- `cargo init` e `Cargo.toml` com todas as dependências
- `table.rs`: `Cell`, `Sheet`, `TableData`
- Trait `Reader` + factory `reader_for()`
- **11 testes passando**

---

### ✅ Milestone 2 — Leitores de Arquivo
**Concluído em:** 2026-05-01

- `CsvReader`: detecção automática de delimitador (`,` `;` `\t`), BOM UTF-8
- `XlsxReader`: via calamine, múltiplas sheets
- `XlsReader`: via calamine, formato legado
- Fixtures de teste gerados via Python
- **16 testes passando** (7 CSV + 3 XLSX + 1 XLS + 5 factory)

---

### ✅ Milestone 3 — Estado da Aplicação
**Concluído em:** 2026-05-01

- `App` struct com scroll, cursor, navegação de sheets
- Busca inline com highlight e ciclo de resultados
- **14 testes passando**

---

### ✅ Milestone 4 — Renderização TUI
**Concluído em:** 2026-05-01

- Layout: header (arquivo + tabs de sheets) + tabela + status bar
- Tabela com scroll virtual e largura de coluna automática
- Highlight de linha (`CURSOR_ROW_BG`) e célula ativa (`ACTIVE_CELL_BG` amarelo ouro)
- Status bar: `Ln x/y | Col x/y | NomeColuna › valor da célula`
- Help overlay (`?`)
- Evento de teclado mapeado estilo vim (`hjkl`, `g/G`, `Tab`, `/`)

---

### 🔄 Milestone 5 — Edição estilo Vim
**Em planejamento**

#### Novas funcionalidades
- Modo `Insert`: editar célula com buffer visual, confirmar/cancelar
- Modo `Command`: `:w`, `:q`, `:wq`, `:q!`
- Sequências de teclas: `gg`, `dd`, `yy` (pending key buffer)
- Operações: inserir linha (`o`/`O`), deletar (`dd`), copiar (`yy`), colar (`p`/`P`), limpar célula (`x`)
- Undo/Redo: pilha de ações com `u` / `Ctrl+R`
- Indicador de modificado `[+]` no header
- Escritores: `CsvWriter`, `XlsxWriter` (via `rust_xlsxwriter`)

#### Testes planejados
```
app::tests::insert_mode_updates_buffer
app::tests::insert_mode_confirm_changes_cell
app::tests::insert_mode_cancel_restores_cell
app::tests::gg_goes_to_first_row
app::tests::dd_deletes_row
app::tests::yy_copies_row
app::tests::p_pastes_row_below
app::tests::P_pastes_row_above
app::tests::x_clears_cell
app::tests::undo_reverts_edit
app::tests::undo_reverts_delete
app::tests::redo_reapplies_edit
app::tests::dirty_flag_set_on_edit
app::tests::dirty_flag_cleared_on_save
writer::csv::tests::writes_simple_csv
writer::csv::tests::roundtrip_csv
writer::xlsx::tests::writes_single_sheet
writer::xlsx::tests::roundtrip_xlsx
```

---

### 📋 Milestone 6 — README e Publicação
**Pendente**

- README.md completo: instalação, uso, screenshots, tabela de atalhos
- `cargo clippy` sem warnings
- `cargo test` 100% passando
- Release binary via `cargo build --release`

---

## Convenções de Código

- Sem `unwrap()` fora de testes
- Erros propagados com `?` e tipo `AppError`
- Módulos com `#[cfg(test)]` inline para unit tests
- Integração em `tests/` com fixtures reais
- Sem comentários óbvios; apenas invariantes não-triviais

---

## Critérios de Aceite

### Leitor CSV
- [x] Detecta automaticamente delimitador (`,` `;` `\t`)
- [x] Suporta aspas e campos com quebra de linha
- [x] Suporta UTF-8 com e sem BOM

### Leitor XLSX/XLS
- [x] Lê todas as sheets disponíveis
- [x] Preserva tipos: texto, número, data, booleano
- [x] Não falha em células vazias ou mergeadas

### Editor
- [ ] Edição confirma com `Enter`, cancela com `Esc`
- [ ] `dd` com undo funciona corretamente
- [ ] Arquivo modificado mostra `[+]` no header
- [ ] `:w` salva sem sair; `:q` bloqueia se houver mudanças não salvas

### TUI
- [x] Funciona em terminais 80x24 e maiores
- [x] Não quebra com dados Unicode (emoji, acentos, CJK)
- [ ] Scroll suave em arquivos com 100k+ linhas
