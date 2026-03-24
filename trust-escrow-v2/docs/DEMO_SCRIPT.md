# Demo Script - Trust Work Escrow v2 (Hackathon)

## 🎤 Script de Presentación (5 minutos)

### 1. Introducción (30 segundos)
"Buenas noches, presentamos **Trust Work Escrow v2**, un protocolo de escrow descentralizado en Solana para trabajos freelance."

### 2. Problema (30 segundos)
"El problema actual: freelancers temen no recibir pago, clientes temen mala calidad. Plataformas cobran 20% comisión y disputas son subjetivas."

### 3. Solución (1 minuto)
"Nuestra solución:
- **Smart Contract** en Solana que bloquea fondos hasta aprobación
- **Sistema de árbitros** para disputas transparentes  
- **Milestones** para pagos parciales
- **Comisiones 0-1%** vs 20% tradicional"

### 4. Demo Técnica (2 minutos)

**Paso 1: Mostrar deployment**
```bash
# Mostrar programa desplegado
solana program show 28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA
# Explicar: "Programa desplegado en devnet, 31 instrucciones"
```

**Paso 2: Ejecutar CLI**
```bash
cd trust-escrow-v2
./target/release/trust-escrow status
# Explicar: "CLI conectada a devnet, programa ejecutable"
```

**Paso 3: Mostrar explorer**
```
# Abrir navegador con:
https://explorer.solana.com/address/28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA?cluster=devnet
# Explicar: "Transacciones visibles en explorador"
```

**Paso 4: Ejecutar TUI**
```bash
./target/debug/trust-escrow-tui
# Explicar: "Interfaz terminal con dashboard, navegación por teclado"
```

### 5. Stack Técnico (30 segundos)
"Tecnologías:
- **Solana + Anchor** para smart contracts
- **Rust** para SDK, CLI, TUI
- **Arquitectura hexagonal** para maintainability
- **9,000+ líneas** de código en 38 horas"

### 6. Cierre (30 segundos)
"Próximos pasos: Auditoría de seguridad, tests completos, mainnet. ¡Gracias!"

---

## 🎯 Comandos para Demo Rápida

### Pre-Demo (Preparar todo)
```bash
# 1. Verificar wallet
solana balance

# 2. Verificar programa
solana program show 28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA

# 3. Compilar TUI si es necesario
cd trust-escrow-v2 && cargo build -p trust-escrow-tui
```

### Durante Demo
```bash
# Mostrar status CLI
./target/release/trust-escrow status

# Mostrar balance
./target/release/trust-escrow payment balance

# Ejecutar demo completo
./demo.sh

# Ejecutar TUI
./target/debug/trust-escrow-tui
```

---

## 📋 Checklist Pre-Demo

- [ ] Terminal configurado con fuente grande
- [ ] Wallet con SOL de devnet
- [ ] Binarios compilados
- [ ] Explorer abierto en navegador
- [ ] TUI probado
- [ ] CLI probada

---

## 💡 Tips para Presentación

1. **Hablar claro y pausado**
2. **Mostrar código real** (no screenshots)
3. **Conectar con el problema** (freelance economy)
4. **Destacar diferenciadores** (comisiones bajas, transparencia)
5. **Llamar a la acción** (auditoría, mainnet)

---

**¡Recordar! El deadline es HOY 23:30. Practicar 2-3 veces antes.**