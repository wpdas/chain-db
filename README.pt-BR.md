# ChainDB

[English](README.md) | [Português](#português)

# Português

ChainDB é um banco de dados simples que mantém um histórico de alterações, permitindo acompanhar como seus dados evoluem ao longo do tempo.

## Características

- 📊 Rastreamento completo do histórico de alterações
- 🔒 Armazenamento seguro com criptografia AES-256-GCM
- 🔑 Derivação de chave baseada em senha
- 🌐 API REST para manipulação de dados
- 📚 Suporte para múltiplos bancos de dados
- 📁 Estrutura de arquivos organizada com segmentação eficiente
- ⚡ Operações rápidas de leitura e escrita
- 🔄 Controle de versão e rastreamento automático de dados

## Segurança e Criptografia

O ChainDB utiliza um sistema robusto de criptografia para proteger seus dados:

1. **Sistema de Derivação de Chave**

   - Cada senha gera uma chave única de criptografia usando SHA-256
   - A mesma senha sempre gera a mesma chave
   - Senhas diferentes geram chaves diferentes
   - A chave de criptografia nunca é armazenada, apenas derivada quando necessário

2. **Criptografia de Dados**

   - Todos os dados são criptografados usando AES-256-GCM
   - Cada registro é criptografado individualmente
   - Um nonce único é usado para cada operação de criptografia
   - O nonce é armazenado junto com os dados criptografados

3. **Gerenciamento de Senha**

   - Senhas nunca são armazenadas em texto puro
   - Mudar a senha requer recriptografar todos os dados
   - A mudança de senha é uma operação atômica (tudo ou nada)
   - Se uma senha for perdida, os dados não podem ser recuperados

4. **Processo de Armazenamento**
   - Ao salvar dados:
     1. Os dados são convertidos para formato JSON (legível)
     2. O JSON é criptografado usando a chave derivada (AES-256-GCM)
     3. Os dados criptografados são salvos em arquivos .cdb
   - Ao ler dados:
     1. Os dados criptografados são lidos dos arquivos .cdb
     2. Os dados são descriptografados usando a chave derivada
     3. O JSON é convertido de volta para a estrutura de dados

## Instalação

Adicione ChainDB ao seu `Cargo.toml`:

```toml
[dependencies]
chain-db-v2 = "0.1.0"
```

## Início Rápido

Aqui está um exemplo básico de como usar o ChainDB:

```rust
use chain_db_v2::{ChainDB};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct GreetingTable {
    greeting: String,
}

impl GreetingTable {
    fn new() -> Self {
        GreetingTable {
            greeting: String::from("Olá"),
        }
    }
}

#[tokio::main]
async fn main() {
    // 1. Conectar ao banco de dados
    let connection = ChainDB::connect(
        "test-db",    // Nome do banco de dados
        "root",       // Usuário
        "1234",       // Senha
    ).await;
    let db = &connection.db;

    // 2. Criar/Obter uma tabela
    let mut greeting_table = db.get_table("greeting", GreetingTable::new).await;

    // 3. Ler valor atual
    println!("{:?}", greeting_table.table.greeting);

    // 4. Atualizar último registro
    greeting_table.table.greeting = String::from("Olá Mundo!");
    greeting_table.update_data().await;

    // 5. Criar novo registro
    greeting_table.table.greeting = String::from("Ei!");
    greeting_table.persist_data().await;

    // 6. Obter histórico (últimos 50 registros)
    let history = greeting_table.get_data_history(50).await;
    println!("{:?}", history);
}
```

## Testando a API

1. Inicie o servidor:

```bash
cargo run
```

O servidor será iniciado em `http://localhost:2818`

### Autenticação

Todos os endpoints da API (exceto criação de banco de dados) requerem autenticação usando Basic Auth. O cabeçalho de autenticação deve ser construído da seguinte forma:

1. Crie uma string no formato: `nome_do_banco:usuario:senha`
   - Exemplo: `test_db:root:1234`
2. Codifique esta string em Base64
   - Exemplo: `dGVzdF9kYjpyb290OjEyMzQ=`
3. Adicione o prefixo "Basic " à string codificada
   - Cabeçalho final: `Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=`

Você pode gerar a string Base64 usando várias ferramentas:

```bash
# Usando linha de comando
echo -n "test_db:root:1234" | base64

# Usando Python
python3 -c "import base64; print(base64.b64encode('test_db:root:1234'.encode()).decode())"
```

### Endpoints da API

1. Criar um banco de dados:

```bash
curl -X POST http://localhost:2818/api/v1/database/create \
  -H "Content-Type: application/json" \
  -d '{"name": "test_db", "user": "root", "password": "1234"}'
```

2. Conectar ao banco de dados:

```bash
curl -X POST http://localhost:2818/api/v1/database/connect \
  -H "Content-Type: application/json" \
  -d '{"name": "test_db", "user": "root", "password": "1234"}'
```

A resposta incluirá um token de autenticação que deve ser usado nas chamadas subsequentes da API:

```json
{
  "success": true,
  "data": "Basic dGVzdF9kYjpyb290OjEyMzQ="
}
```

Use este token no cabeçalho Authorization para todas as solicitações subsequentes:

```bash
Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=
```

3. Mudar senha:

```bash
curl -X POST http://localhost:2818/api/v1/database/change-password \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test_db",
    "user": "root",
    "old_password": "1234",
    "new_password": "nova_senha"
  }'
```

**Importante**: Ao mudar a senha, todos os dados precisam ser recriptografados com a nova chave. Este processo:

- Pode demorar mais para bancos de dados maiores
- É atômico (ou todos os dados são recriptografados ou nenhum)
- Não pode ser interrompido com segurança
- Deve ser feito durante períodos de baixo uso
- Cria um backup automático de todos os dados antes de fazer alterações (em uma pasta chamada `bkp_AAAAMMDD_HHMMSS`)
- Verifica o espaço em disco disponível antes de iniciar (requer pelo menos 2x o tamanho atual do banco de dados)
- Reverte para o estado original se ocorrer algum erro durante o processo

4. Atualizar dados da tabela:

```bash
curl -X POST http://localhost:2818/api/v1/table/greetings/update \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=" \
  -d '{"data": {"greeting": "Olá, Mundo!"}}'
```

5. Obter dados atuais da tabela:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

6. Persistir novo registro:

```bash
curl -X POST http://localhost:2818/api/v1/table/greetings/persist \
  -H "Content-Type: application/json" \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ=" \
  -d '{"data": {"greeting": "Novo Registro!"}}'
```

7. Obter histórico:

```bash
curl -X GET http://localhost:2818/api/v1/table/greetings/history?limit=10 \
  -H "Authorization: Basic dGVzdF9kYjpyb290OjEyMzQ="
```

## Como Funciona

1. **Criação do Banco de Dados**:

   - Cria uma pasta com o nome do banco de dados
   - Armazena configurações em `config.cdb`
   - Armazena credenciais de forma segura

2. **Gerenciamento de Tabelas**:

   - Cada tabela tem seu próprio diretório
   - Os dados são armazenados em blocos de 1000 registros por arquivo
   - Gerenciamento e indexação automática de arquivos

3. **Operações**:
   - `update_data()`: Atualiza o último registro
   - `persist_data()`: Cria um novo registro
   - `get_data_history(n)`: Recupera os últimos N registros

## Segurança

- Todos os dados são criptografados em repouso
- Autenticação é necessária para todas as operações
- Senhas nunca são armazenadas em texto puro
- Chaves de criptografia são derivadas de senhas usando funções seguras

## Estrutura de Arquivos

```
seu-banco-de-dados/
├── config.cdb             # Configuração criptografada do banco
├── table1/               # Diretório da primeira tabela
│   ├── metadata.cdb     # Metadados criptografados da tabela
│   ├── data_0.cdb      # Primeiro bloco de dados
│   ├── data_1.cdb      # Segundo bloco de dados
│   └── ...
└── table2/              # Diretório da segunda tabela
    ├── metadata.cdb
    └── ...
```

## Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo LICENSE para detalhes.
