# 🦀 Rinha de Backend · Rust

![Rust](https://img.shields.io/badge/Rust-2021_Edition-000000?logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-0.7-555555)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15-4169E1?logo=postgresql&logoColor=white)
![Status](https://img.shields.io/badge/Status-Em_desenvolvimento-yellow)

API de cadastro e consulta de pessoas em **Rust**, baseada no desafio da [Rinha de Backend — edição 2023 Q3](https://github.com/zanfranceschi/rinha-de-backend-2023-q3). O projeto explora HTTP assíncrono com Axum e Tokio, validação de dados com Serde e persistência em PostgreSQL com SQLx.

> Os commits deste repositório são de julho de 2024, mas o domínio e os endpoints correspondem ao desafio de pessoas da edição de 2023. A integração com PostgreSQL está incompleta e a revisão atual apresenta bloqueios de compilação descritos abaixo.

## ⚙️ Tecnologias

| Tecnologia | Papel no projeto |
| --- | --- |
| Rust · edição 2021 | Linguagem da aplicação |
| Axum 0.7 | Rotas, extração de parâmetros e respostas HTTP |
| Tokio | Runtime assíncrono e primitivas de sincronização |
| Serde / serde_json | Serialização, desserialização e validação do JSON |
| SQLx 0.7 | Pool de conexões e consultas SQL parametrizadas |
| PostgreSQL 15 | Persistência, restrições de integridade e estrutura de busca |
| UUID v7 | Geração de identificadores de pessoas |
| Docker Compose | Ambiente local do banco de dados |

As dependências estão declaradas em [Cargo.toml](Cargo.toml), com resolução registrada em [Cargo.lock](Cargo.lock).

## 🧩 Organização e implementação

| Arquivo | Responsabilidade |
| --- | --- |
| `src/main.rs` | Modelos, validações, handlers, rotas e inicialização do servidor |
| `src/persistance.rs` | Repositório PostgreSQL para criação, consulta, busca e contagem |
| `db/v0001__create_people.sql` | Tabela, índice de busca e trigger de notificação |
| `docker-compose-dev.yml` | Serviço PostgreSQL para desenvolvimento |
| `cargo/config.toml` | Flag de compilação registrada no projeto |

Os handlers ainda utilizam um `HashMap<Uuid, Person>` compartilhado por `Arc<Mutex<_>>`. Em paralelo, o repositório SQL já contém consultas para PostgreSQL, mas a inicialização tenta injetá-lo em rotas que esperam o estado em memória. Essa transição precisa ser concluída.

O schema do banco inclui:

- Chave primária UUID e apelido único.
- Data de nascimento como `DATE` e tecnologias como array de strings.
- Coluna `search` gerada a partir do nome, apelido e tecnologias.
- Extensão `pg_trgm` e índice GiST sobre o texto de busca.
- Trigger que publica o registro inserido no canal `person_created` via `PG_NOTIFY`. Ainda não há consumidor desse canal no código Rust.

## 🔌 Endpoints

As rotas abaixo estão declaradas no código. O comportamento descrito corresponde aos handlers existentes, que dependem da correção dos bloqueios de compilação para serem executados.

| Método | Rota atual | Implementação |
| --- | --- | --- |
| `POST` | `/pessoas` | Cria uma pessoa em memória e retorna `201` com o registro em JSON |
| `GET` | `/pessoa/:id` | Consulta em memória por UUID; retorna JSON ou `404` |
| `GET` | `/pessoas` | Placeholder: retorna o texto `Busca` |
| `GET` | `/contagem-pessoas` | Retorna a quantidade de registros em memória |

**Atenção:** o desafio original exige `GET /pessoas/:id`, no plural. A rota atual usa `/pessoa/:id`.

### Payload de criação

```json
{
  "nome": "Guilherme Freitas",
  "apelido": "guilherme",
  "nascimento": "2000-01-15",
  "stack": ["Rust", "PostgreSQL"]
}
```

| Campo | Validação atual no código Rust |
| --- | --- |
| `nome` | String obrigatória, com até 100 bytes |
| `apelido` | String obrigatória, com até 32 bytes |
| `nascimento` | Data obrigatória no formato `AAAA-MM-DD` |
| `stack` | Opcional ou `null`; quando informado, array de strings de até 32 bytes cada |

Os limites usam `String::len()`, que conta bytes UTF-8. A unicidade do apelido está definida no banco, mas não é validada pelo handler em memória.

## 🚀 Preparação do ambiente

Requisitos: Rust e Cargo, Docker e Docker Compose. Não há versão de toolchain fixada no repositório.

```bash
git clone https://github.com/GuilhermeVgl/rinha.git
cd rinha
docker compose -f docker-compose-dev.yml up -d
```

O Compose inicia **somente o PostgreSQL**, na porta `5432`, e monta os scripts de `db/` em `/docker-entrypoint-initdb.d`. Esses scripts são executados durante a primeira inicialização de um diretório de dados vazio.

Credenciais locais: usuário `rinha`, senha `rinha` e banco `rinha`.

### Configuração da aplicação

| Variável | Valor previsto no código | Estado atual |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://rinha:rinha@localhost:5432/rinha` | O literal padrão contém um erro de sintaxe a corrigir |
| `DATABASE_POOL` | `30` | Lida no início, mas o repositório fixa o pool em 5 conexões |
| `PORT` | `9999` | Lida no início, mas o listener está fixado em `0.0.0.0:3000` |

Após resolver as pendências de compilação e integração:

```bash
export DATABASE_URL='postgres://rinha:rinha@localhost:5432/rinha'
cargo check --locked
cargo run --release --locked
```

Enquanto o bind permanecer como está no código, o endereço será `http://localhost:3000`.

## 🛠️ Pendências conhecidas

- **Compilação:** remover o prefixo indevido `gg` do literal de conexão em `main.rs`.
- **Conexão:** alinhar a chamada de `PostgresRepository::connect` com sua assinatura e retorno. Atualmente, a chamada passa dois argumentos e usa `.unwrap()` após o `await`, enquanto o método recebe uma `String` e retorna `Self`.
- **Estado das rotas:** integrar os handlers ao repositório PostgreSQL, substituindo o estado em memória.
- **Busca:** implementar a leitura de `?t=` e alinhar a consulta ao schema. A consulta atual usa `to_tsquery('people', $1)`, mas o script não cria essa configuração de busca e define `search` como texto com índice de trigramas.
- **Contagem:** alinhar o tipo de retorno ao `BIGINT` produzido por `COUNT(*)` no PostgreSQL, usando, por exemplo, `i64`.
- **Contrato HTTP:** ajustar a rota de consulta para o plural, adicionar o header `Location` na criação e tratar erros conforme as regras do desafio.
- **Configuração:** aplicar `PORT` e `DATABASE_POOL` efetivamente. O arquivo `cargo/config.toml` também precisa ser revisado: a localização convencional para descoberta automática pelo Cargo é `.cargo/config.toml`.
- **Validação e entrega:** adicionar testes automatizados, testes de carga e configuração de execução da API em containers.

O repositório ainda não inclui resultados de benchmark, testes automatizados ou configuração completa de submissão à competição.

## 📚 Referências

- [Rinha de Backend — 2023 Q3](https://github.com/zanfranceschi/rinha-de-backend-2023-q3)
- [Contrato da API e regras do desafio](https://github.com/zanfranceschi/rinha-de-backend-2023-q3/blob/main/INSTRUCOES.md)

---

Desenvolvido por [Guilherme Freitas](https://github.com/GuilhermeVgl).
