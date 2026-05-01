# kikito-spreadsheet

> Visualizador e editor de planilhas Excel e CSV no terminal — construído com Rust e ratatui.

```
 📊 clientes.xlsx  │  Planilha1
┌──────────────────────────────────────────────────────────────────────────┐
│ ID      Nome                 Nascimento   Cidade        Cargo            │
│ 1001    João da Silva        1990-06-08   São Paulo     Analista         │
│▓▓1002▓▓▓▓Maria Oliveira▓▓▓▓▓1992-03-15▓▓▓Campinas▓▓▓▓▓Desenvolvedora▓▓│  ← célula ativa
│ 1003    Carlos Souza         1988-03-15   Rio de Janeiro Gerente         │
│ 1004    Ana Paula Ferreira   1995-11-22   Curitiba      Designer         │
└──────────────────────────────────────────────────────────────────────────┘
 Ln 3/1000  Col 2/6  │  Nome › Maria Oliveira   -- NORMAL --  [?] ajuda
```

---

## Funcionalidades

- **Leitura** de `.xlsx`, `.xls` e `.csv` (detecção automática de delimitador `,` `;` `\t`)
- **Edição** estilo Vim — modos Normal, Insert e Command
- **Escrita** de volta para `.xlsx` e `.csv`
- Navegação por teclado com atalhos vim (`hjkl`, `gg`, `G`, `dd`, `yy`...)
- Busca inline com `/` e navegação entre resultados com `n`/`N`
- Ir para linha com `:número`
- Undo/Redo ilimitado (`u` / `Ctrl+R`)
- Suporte a múltiplas sheets com navegação por `Tab`
- Highlight da célula ativa, linha selecionada e resultados de busca
- Status bar com nome da coluna e valor da célula atual
- Suporte a Unicode, acentos e caracteres especiais
- Indicador `[+]` no header quando há modificações não salvas

---

## Instalação

### Pré-requisitos

- [Rust](https://rustup.rs/) 1.85 ou superior

### Compilar do código-fonte

```bash
git clone https://github.com/demostenex/kikito-spreadsheet
cd kikito-spreadsheet
cargo build --release
```

O binário estará em `target/release/excel-tui`.

### Uso direto com cargo

```bash
cargo run -- arquivo.xlsx
cargo run -- dados.csv
cargo run -- planilha.xls
```

---

## Uso

```bash
excel-tui <arquivo>
```

**Exemplos:**

```bash
excel-tui relatorio.xlsx
excel-tui clientes.csv
excel-tui legado.xls
```

---

## Atalhos de Teclado

### Navegação — Modo Normal

| Tecla | Ação |
|-------|------|
| `j` / `↓` | Linha abaixo |
| `k` / `↑` | Linha acima |
| `h` / `←` | Coluna à esquerda |
| `l` / `→` | Coluna à direita |
| `gg` | Primeira linha |
| `G` | Última linha |
| `0` | Primeira coluna |
| `$` | Última coluna |
| `Tab` | Próxima sheet |
| `Shift+Tab` | Sheet anterior |

### Edição — Modo Normal

| Tecla | Ação |
|-------|------|
| `i` | Editar célula atual |
| `o` | Nova linha abaixo + entrar em Insert |
| `O` | Nova linha acima + entrar em Insert |
| `dd` | Deletar linha atual |
| `yy` | Copiar linha atual |
| `p` | Colar linha abaixo |
| `P` | Colar linha acima |
| `x` | Limpar conteúdo da célula |
| `u` | Desfazer (undo) |
| `Ctrl+R` | Refazer (redo) |
| `Ctrl+S` | Salvar |

### Modo Insert

| Tecla | Ação |
|-------|------|
| Qualquer caractere | Adiciona ao buffer de edição |
| `Backspace` | Apaga último caractere |
| `Enter` | Confirma edição |
| `Esc` | Cancela edição |

### Modo Command (`:`)|

| Comando | Ação |
|---------|------|
| `:w` | Salvar arquivo |
| `:q` | Sair (bloqueado se houver mudanças) |
| `:wq` | Salvar e sair |
| `:q!` | Sair sem salvar |
| `:42` | Ir para a linha 42 |

### Busca

| Tecla | Ação |
|-------|------|
| `/` | Entrar no modo busca |
| `n` | Próximo resultado (Normal mode) |
| `N` | Resultado anterior (Normal mode) |
| `Esc` / `Enter` | Sair da busca (resultados permanecem) |

### Geral

| Tecla | Ação |
|-------|------|
| `?` | Abrir/fechar ajuda |
| `q` | Sair |

---

## Formatos Suportados

| Formato | Leitura | Escrita |
|---------|---------|---------|
| `.xlsx` | ✅ | ✅ |
| `.xls`  | ✅ | ✅ (salva como xlsx) |
| `.csv`  | ✅ | ✅ |

**CSV:** detecta automaticamente o delimitador (`,` `;` `\t`) e suporta UTF-8 com BOM.

---

## Desenvolvimento

O projeto foi desenvolvido com TDD — todos os testes foram escritos antes da implementação.

```bash
# rodar todos os testes
cargo test

# build de release
cargo build --release

# verificar qualidade do código
cargo clippy
```

**Stack:**

| Crate | Função |
|-------|--------|
| `ratatui` | Renderização TUI |
| `crossterm` | Backend de terminal |
| `calamine` | Leitura de xls/xlsx |
| `rust_xlsxwriter` | Escrita de xlsx |
| `csv` | Leitura e escrita de CSV |
| `thiserror` | Tipos de erro |

---

## Licença

MIT
