# Registro de Alterações

[English](CHANGELOG.md) | [Português](#registro-de-alterações)

## 1.0.0 (2023-11-15)

### Principais Mudanças em Relação à v0.0.3

- **Segurança Aprimorada**: Implementação de criptografia AES-256-GCM para todos os dados armazenados, substituindo o modelo anterior sem criptografia
- **Gerenciamento de Senhas**: Sistema de derivação de chave baseado em senha usando SHA-256, oferecendo maior segurança para acesso aos dados
- **Backup Automático**: Adição de sistema de backup automático durante operações críticas como mudança de senha
- **Verificação de Espaço em Disco**: Implementação de verificações de recursos do sistema antes de operações que exigem espaço adicional
- **Atomicidade de Operações**: Garantia de que operações críticas são atômicas (tudo ou nada) com mecanismo de rollback em caso de falha
- **API REST Completa**: Redesenho da API para oferecer mais funcionalidades e melhor integração com aplicações
- **Estrutura de Arquivos Otimizada**: Reorganização da estrutura de arquivos para melhor desempenho e gerenciamento
- **Segmentação de Dados**: Implementação de segmentação eficiente de dados em blocos de 1000 registros
- **Suporte a Múltiplos Bancos de Dados**: Capacidade de gerenciar múltiplos bancos de dados independentes
- **Foco em Histórico de Dados**: Mantém o conceito de rastreamento de histórico da versão anterior, mas com implementação mais robusta
- **Mudança de Paradigma**: Transição de um modelo inspirado em blockchain para um sistema de banco de dados com histórico mais tradicional e eficiente

### Remoções

- Eliminação do conceito de "unidades" e transferências presentes na versão anterior
- Remoção da abordagem de blocos encadeados no estilo blockchain
- Simplificação do modelo de usuário

### Melhorias Técnicas

- Reescrita completa do código-base para maior eficiência e manutenibilidade
- Melhor tratamento de erros e recuperação de falhas
- Documentação mais abrangente e exemplos de uso
