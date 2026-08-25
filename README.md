# ⚡ Postman da Vida (Tauri Edition) 🚀
**A professional-grade, high-performance API Client built with Tauri, Rust, and React.**

![Premium UI](https://img.shields.io/badge/UI-Elite-blueviolet) ![Status](https://img.shields.io/badge/Status-Phase%206-green) ![Tech](https://img.shields.io/badge/Stack-Tauri%20%7C%20React%20%7C%20Rust-orange)

## ✨ Why Choose Postman da Vida?

- 💎 **Elite UI/UX**: Ultra-modern interface with glassmorphism, Command Palette (Ctrl+P), and Framer Motion.
- 📂 **Local-First Workspace**: Full Git compatibility by storing everything in readable JSON on your drive.
- 📑 **Multi-Protocol Hub**: Support for REST, GraphQL, WebSocket, and gRPC (Mock Hub).
- 📜 **SpecHub (API Design)**: Author and validate OpenAPI 3.0/3.1 specs with real-time Governance Linting.
- 🔐 **Enterprise Security**: Military-grade AES-256-GCM encryption for your Secrets/API Keys at rest.
- ⚡ **Rust-Powered Engine**: Blazing fast request execution with integrated Load Testing (Tokio multi-threading).
- 🔄 **Resilient Sync**: Background synchronization with an Offline Queue for unreliable network environments.
- 📦 **Cross-Platform**: Ready-to-go installers for Windows, macOS, and Linux.
- 🌓 **Dynamic Themes**: Switch between Elite Dark and Professional Light modes instantly.

## 🏗️ Architecture & Philosophy

Built on **Clean Architecture** and SOLID principles:
- **Core (Rust)**: Entity-driven logic using `reqwest` and `QuickJS` for maximum performance.
- **Frontend (React 19)**: Atomic components using `zustand` for ultra-lean state management.
- **Security**: Hardware-level isolation through Tauri's security model.

## 🛠️ Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/)
- [Node.js](https://nodejs.org/)

### Installation
1. Clone the repository.
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

## 📜 Automation & Scripting

The app features a unified **JavaScript Sandbox** (pm API) to automate your testing workflows:

```javascript
// Post-request Tests
pm.test("Status is 201 Created", () => {
    pm.response.to.have.status(201);
});

pm.environment.set("userId", pm.response.json().id);
```

### Test Scripts
Run logic after receiving a response. Access the `pm` object to validate data.

**Example Test:**
```javascript
pm.test("Status is 200 OK", () => {
    expect(pm.response.status).to.equal(200);
});

pm.test("Content-Type is JSON", () => {
    const contentType = pm.response.headers["content-type"];
    expect(contentType).to.include("application/json");
});

pm.test("Check User ID", () => {
    const data = pm.response.json();
    expect(data.id).to.equal(1);
});
```

### Environment Manipulation
You can dynamically update your environment variables from any script:

```javascript
// Pre-request: Set a timestamp for use in headers
pm.environment.set("request_time", new Date().toISOString());

// Test: Capture a token from the response for subsequent requests
const token = pm.response.json().token;
pm.environment.set("auth_token", token);
```

### Supported Assertions
- `expect(val).to.equal(other)`
- `expect(val).to.include(substring)`
- `expect(val).to.be.a("type")` (e.g., "object", "string")

Results are displayed in the **Test Results** tab in the response area with pass/fail indicators.

---

## ⚡ O Poder da Automação (CI/CD Local)

O **Postman da Vida** não é apenas um cliente HTTP, é uma ferramenta de teste completa. Veja como você pode automatizar fluxos complexos:

### Exemplo: Fluxo de Autenticação & Validação de Dados

1. **Pre-request Script (Configura o ambiente):**
```javascript
// Gera um ID único para o teste e salva no ambiente
const testId = "test_" + Math.floor(Math.random() * 1000);
pm.environment.set("current_test_id", testId);
console.log("Iniciando teste: " + testId);
```

2. **Test Script (Valida a Resposta):**
```javascript
// 1. Valida Status Code
pm.test("Status é 200 OK", () => {
    expect(pm.response.status).to.equal(200);
});

// 2. Valida Estrutura do JSON
const response = pm.response.json();
pm.test("Retornou o usuário correto", () => {
    expect(response.name).to.equal("Leanne Graham");
    expect(response.id).to.be.a("number"); // Validação de tipo
});

// 3. Captura dado para a próxima requisição
if (pm.response.status === 200) {
    const userCity = response.address.city;
    pm.environment.set("last_city_checked", userCity);
    console.log("Cidade capturada: " + userCity);
}
```

3. **Collection Runner:**
Abra o **Collection Runner**, selecione sua pasta de testes e execute. Você receberá um relatório detalhado com a contagem de sucessos/falhas em milissegundos.

---
Built with ❤️ by Antigravity AI.
# postman-tauri-clone
