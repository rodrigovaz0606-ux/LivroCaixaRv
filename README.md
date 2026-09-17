# Livro Caixa RV

Aplicação web para controlar as entradas, saídas e o saldo anual de um mercado.

## Como executar

Requer Rust instalado. Na pasta do projeto:

```powershell
cargo run
```

Acesse http://127.0.0.1:3000. O banco SQLite `livro_caixa.db` e suas tabelas são criados automaticamente na primeira execução.

## Recursos

- Seleção e organização por exercício (ano)
- Saldo final transportado automaticamente para o exercício seguinte
- Possibilidade de ajustar manualmente o saldo inicial de qualquer ano
- Cadastro, edição, exclusão e pesquisa de lançamentos
- Entradas, saídas e saldo final calculados automaticamente
- Dados monetários armazenados em centavos, evitando erros de arredondamento
- Interface adaptada para computador e celular

## Configuração opcional

- `DATABASE_URL`: endereço do banco SQLite (padrão: `sqlite://livro_caixa.db?mode=rwc`)
- `PORT`: porta HTTP (padrão: `3000`)
