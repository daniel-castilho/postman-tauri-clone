# Project Progress Tracker

Este documento serve como um mapa de estado global do projeto. Qualquer Agente de IA assumindo o projeto deve consultar este arquivo (junto com o `AGENTS.md`) para entender em que fase de desenvolvimento nos encontramos.

## Current Stage: **Fase 18 (Headless Automation & CLI)** - Em Andamento 🚀

---

### Milestones Atingidos (Fase 8-14 Completa ✅ - Plataforma Base)
- [x] **Multi-Protocolo**: GraphQL, WebSocket, gRPC (Mock Hub).
- [x] **Cookie Manager**: Persistência de sessão automática.
- [x] **IA-Native**: Test Generator e Explainaions via Gemini.
- [x] **Load Testing Engine**: Testes multi-threaded em Rust (Tokio).
- [x] **Collaborative Sync**: Sync em tempo real via Tauri Events com `SyncQueue`.
- [x] **Workspace Member Management**: Gestão de acessos com controle granular.

### Milestones Atingidos (Fase 15 Completa ✅ - DX & Power Tools)
- [x] **Global Command Palette (Ctrl + P)**: Busca instantânea em todo o workspace (Requests, Envs, Actions).
- [x] **Keyboard Shortcuts Engine**: Atalhos premium para `Send`, `Save` e Navegação.
- [x] **Quick Environment Switcher**: Alternância instantânea via teclado (Ctrl + 1-9).

### Milestones Atingidos (Fase 17 Completa ✅ - Enterprise Security)
- [x] **Local Secrets Encryption (AES-256-GCM)**: Proteção de segredos em nível militar no disco via Rust.
- [x] **OfflineSync Queue**: Resiliência extrema para sincronização em ambientes instáveis.
- [x] **Transparent Security**: Descriptografia automática e segura de variáveis locais.

### Milestones Atingidos (Fase 20 Completa ✅ - SpecHub / API Design)
- [x] **API Design Hub**: Espaço dedicado para autoria de especificações OpenAPI 3.0/3.1.
- [x] **Governance Linter**: Validação de padrões de design em tempo real no backend (Rust).
- [x] **Unified Context Switching**: Interface híbrida para Design e Execução.

---

## 🚀 Próximas Atividades (Rumo ao Postman Killer)

### Fase 18: Headless Automation (O CLI) (PRÓXIMO FOCO)
- [ ] **Tauri CLI Commands**: Adicionar sub-comandos ao binário para execução de coleções via terminal.
- [ ] **JSON/JUnit Reporting**: Geração de relatórios de testes para pipelines CI/CD.
- [ ] **GitHub Actions Integration**: Template oficial de automação.

### Fase 19: Advanced Scripting & npm Integration
- [ ] **Dynamic Dependency Resolver**: Possibilidade de importar pacotes npm nos scripts (QuickJS sandbox expandida).
- [ ] **In-App Package Manager**: Gestão visual de bibliotecas externas.

### Fase 16: Advanced Collaboration & Presence
- [ ] **Live Avatars/Presence**: Identificação visual de membros ativos no workspace core.
- [ ] **Conflict Resolution UI**: Sistema de merge/diff quando houver edições simultâneas (Advanced CRDT).

---

### 📐 Diretivas de Arquitetura (Atenção!)
- [ ] **Strategy Pattern**: Sempre que houver acúmulo de `match` complexos em rede ou lógica de negócio, refatorar para Traits + Enums tipados para extensibilidade.

### Arquitetura Atual
* **Frontend**: React 19 + Zustand + Monaco Editor + Lucide Icons + Framer Motion.
* **Backend**: Tauri v2 (Rust). Encryption via AES-256-GCM.
* **IA**: Gemini 1.5 Series (via logic layer).
* **Mock**: Axum Server integrado no Rust.
* **SpecHub**: OpenAPI Design & Governance Engine.
* **Sync**: Background robusto com `Offline Queue` e detecção de conectividade.
* **Persistência**: Local-First via repositórios Fs (Collection, Environment, Design, Globals).

---
_Última Atualização: Janeiro 2024. Fases 15, 17 e 20 finalizadas. Foco agora na automação terminal (CLI)._
