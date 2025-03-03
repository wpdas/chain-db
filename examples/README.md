# Exemplos de Uso do ChainDB

Este diretório contém exemplos de uso do ChainDB, tanto usando a biblioteca diretamente quanto através da API REST.

## Exemplos Disponíveis

### 1. Uso Direto da Biblioteca (`example_usage.rs`)

Demonstra como usar o ChainDB diretamente em seu código Rust:

- Criação de banco de dados
- Definição de schema via struct
- Manipulação de dados em memória
- Atualização de registros existentes (update)
- Criação de novos registros (persist)

Para executar:

```bash
cargo run --example example_usage
```

### 2. Uso via API REST

A API REST pode ser acessada usando qualquer cliente HTTP. Aqui estão alguns exemplos usando `curl`:

#### Criar um Banco de Dados

```bash
curl -X POST http://localhost:2818/api/v1/database/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "mydb",
    "user": "root",
    "password": "1234"
  }'
```

#### Criar Primeiro Registro

A tabela é criada automaticamente na primeira persistência:

```bash
curl -X POST http://localhost:2818/api/v1/table/users/persist \
  -H "X-DB-Name: mydb" \
  -H "X-DB-Username: root" \
  -H "X-DB-Password: 1234" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "name": "John Doe",
      "age": 30,
      "email": "john@example.com"
    }
  }'
```

#### Atualizar Último Registro

```bash
curl -X POST http://localhost:2818/api/v1/table/users/update \
  -H "X-DB-Name: mydb" \
  -H "X-DB-Username: root" \
  -H "X-DB-Password: 1234" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "name": "John Doe",
      "age": 31,
      "email": "john.doe@example.com"
    }
  }'
```

#### Criar Novo Registro

```bash
curl -X POST http://localhost:2818/api/v1/table/users/persist \
  -H "X-DB-Name: mydb" \
  -H "X-DB-Username: root" \
  -H "X-DB-Password: 1234" \
  -H "Content-Type: application/json" \
  -d '{
    "data": {
      "name": "John Doe",
      "age": 32,
      "email": "john.doe.new@example.com"
    }
  }'
```

## Operações de Dados

### Update (Atualizar Último Registro)

- Modifica o último registro existente
- Não cria um novo registro na cadeia
- Útil para correções ou atualizações simples

```rust
// Exemplo em Rust
greeting_table.greeting = "Valor atualizado";
greeting_table.update().await?; // Modifica o último registro
```

### Persist (Criar Novo Registro)

- Cria um novo registro na cadeia
- Mantém o histórico completo
- Ideal para registrar mudanças significativas

```rust
// Exemplo em Rust
greeting_table.greeting = "Novo valor";
greeting_table.persist().await?; // Cria novo registro
```

## Schema e Validação

O ChainDB usa um sistema de schema flexível baseado nas estruturas de dados do cliente. Por exemplo:

### Em Rust

```rust
#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
    email: String,
}
```

### Em TypeScript

```typescript
interface User {
  name: string;
  age: number;
  email: string;
}
```

O schema é definido implicitamente pela estrutura usada no cliente, não pelo banco de dados.

## Headers de Autenticação

Todas as requisições à API (exceto criação/conexão de banco) precisam incluir:

```
X-DB-Name: nome_do_banco
X-DB-Username: usuario
X-DB-Password: senha
```

## Formato das Respostas

```json
{
  "success": true,
  "message": "Optional error message",
  "data": {
    // Dados da resposta
  }
}
```
