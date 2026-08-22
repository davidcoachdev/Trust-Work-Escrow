# Trust Work Escrow v2 - Hackathon Demo

## 📋 Resumen Ejecutivo

**Trust Work Escrow v2** es un protocolo de escrow descentralizado en Solana para trabajos freelance, con智能合约 que garantiza pagos justos y resolución de disputas transparentes.

## 🎯 Problema que Resuelve

- Freelancers temen no recibir pago
- Clientes temen no recibir trabajo de calidad
- Disputas subjetivas sin resolución transparente
- Plataformas centralizadas cobran altas comisiones

## 💡 Solución

- **Escrow Automático**: Fondos bloqueados hasta aprobación
- **Disputas con Árbitros**: Sistema descentralizado de resolución
- **Milestones Flexibles**: Pagos parciales por hitos
- **Comisiones Bajas**: 0-1% vs 20% de plataformas tradicionales

## 🏗️ Arquitectura Técnica

### Smart Contract (Solana + Anchor)
- **Program ID**: `28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA`
- **Red**: Devnet (listo para mainnet)
- **Instrucciones**: 31 (user, job, milestone, dispute, etc.)
- **PDAs**: Config, User, Job, Team, Dispute, Milestone

### SDK (Rust)
- **51 operaciones** expuestas
- **Tipos fuertemente tipados**
- **Manejo de errores** completo
- **Integración async/await**

### CLI (Terminal)
- **8 comandos principales**: user, job, milestone, payment, dispute, config, status, airdrop
- **Output formatado**: Texto y JSON
- **Configuración flexible**: Red, wallet, RPC

### TUI (Interfaz Terminal)
- **Layout 3 paneles**: Dashboard, Jobs, Profile
- **Navegación por teclado**: Tab, flechas, letras
- **Pantalla de carga**: Progreso de conexión
- **Datos mock**: Para demo sin conexión

## 🚀 Demo Paso a Paso

### 1. Verificar Despliegue
```bash
solana program show 28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA
```

### 2. Ejecutar Demo CLI
```bash
cd trust-escrow-v2
./demo.sh
```

### 3. Comandos Clave
```bash
# Estado del sistema
./target/debug/trust-escrow status

# Balance de wallet
./target/debug/trust-escrow payment balance

# Configuración
./target/debug/trust-escrow config show

# Listar trabajos
./target/debug/trust-escrow job list
```

### 4. Ejecutar TUI
```bash
./target/debug/trust-escrow-tui
```

## 📊 Estadísticas del Proyecto

| Componente | Estado | Líneas | Tiempo |
|------------|--------|--------|--------|
| Smart Contract | ✅ Deployed | 1,400+ | 12h |
| SDK | ✅ 51 ops | 2,000+ | 8h |
| CLI | ✅ Compila | 1,500+ | 6h |
| TUI | ✅ Funcional | 3,000+ | 8h |
| Documentación | ✅ Completa | 1,000+ | 4h |

**Total**: ~9,000 líneas de código en 38 horas

## 🔗 Enlaces

- **Programa en Explorer**: [Ver en Solana Explorer](https://explorer.solana.com/address/28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA?cluster=devnet)
- **Repositorio**: [GitHub](https://github.com/your-repo)
- **Documentación**: [Docs](./docs/)

## 🎯 Próximos Pasos

1. **Auditoría de seguridad**
2. **Implementar tests completos**
3. **Publicar SDK en crates.io**
4. **Mainnet deployment**
5. **Frontend web (React)**

## 💬 Pitch de 30 Segundos

"Trust Work Escrow es un protocolo de escrow descentralizado en Solana para freelance. Resuelve el problema de confianza entre clientes y freelancers con smart contracts que bloquean pagos hasta aprobación, sistema de disputas con árbitros, y milestones flexibles. Ya desplegado en devnet con CLI y TUI funcionales. Comisiones 0-1% vs 20% de plataformas tradicionales."

---
*Hackathon WayLearn - 23 de Marzo, 2026*