# trust-escrow-v3 Runbooks

[![Surfpool](https://img.shields.io/badge/Operated%20with-Surfpool-gree?labelColor=gray)](https://surfpool.run)

## Available Runbooks

### deployment
Deploy programs

## Getting Started

This repository is using [Surfpool](https://surfpool.run) as a part of its development workflow.

Surfpool provides three major upgrades to the Solana development experience:
- **Surfnet**: A local validator that runs on your machine, allowing you fork mainnet on the fly so that you always use the latest chain data when testing your programs.
- **Runbooks**: Bringing the devops best practice of `infrastructure as code` to Solana, Runbooks allow you to have secure, reproducible, and composable scripts for managing on-chain operations & deployments.
- **Surfpool Studio**: An all-local Web UI that gives new levels of introspection into your transactions.

### Installation

Surfpool installer:

```console
curl -sL https://run.surfpool.run/ | bash
```

Install from source:

```console
# Clone repo
git clone https://github.com/txtx/surfpool.git

# Set repo as current directory
cd surfpool

# Build
cargo surfpool-install
```

### Start a Surfnet

```console
$ surfpool start
```

## Resources

Access tutorials and documentation at [docs.surfpool.run](https://docs.surfpool.run) to understand Surfnets and the Runbook syntax, and to discover the powerful features of surfpool.

Additionally, the [Visual Studio Code extension](https://marketplace.visualstudio.com/items?itemName=txtx.txtx) will make writing runbooks easier.

Our [Surfpool 101 Series](https://www.youtube.com/playlist?list=PL0FMgRjJMRzO1FdunpMS-aUS4GNkgyr3T) is also a great place to start learning about Surfpool and its features:
<a href="https://www.youtube.com/playlist?list=PL0FMgRjJMRzO1FdunpMS-aUS4GNkgyr3T">
  <picture>
    <source srcset="https://raw.githubusercontent.com/txtx/surfpool/main/doc/assets/youtube.png">
    <img alt="Surfpool 101 series" style="max-width: 100%;">
  </picture>
</a>

## Quickstart

Todos los comandos de validación de esta release son exclusivamente localnet.
No se leen keypairs desde archivos versionados y no se permite un endpoint
público. El endpoint debe declararse explícitamente y el preflight debe pasar
antes de cualquier mutación:

```console
$ ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 yarn preflight
```

`txtx.yml` solo declara localnet. Devnet no es un default ni se usa en esta
release; cualquier operación Devnet requiere un override local no versionado,
un endpoint explícito y el mismo preflight antes de ejecutar un runbook.

Después de desplegar en localnet, inicialice o verifique `Config` sin exponer
secretos (las variables siguientes son public keys):

```console
$ TRUST_ESCROW_V3_ADVISOR_PUBKEY=<advisor-public-key> \
  TRUST_ESCROW_V3_TREASURY_PUBKEY=<treasury-public-key> \
  TRUST_ESCROW_V3_ARBITRATION_TREASURY_PUBKEY=<arbitration-treasury-public-key> \
  yarn bootstrap:config
```

El bootstrap usa la autoridad inicial pública compilada en el programa y los
valores explícitos de `advisor`, `treasury`, `arbitration_treasury` y `fee_bps`.
Una `Config` existente se verifica; nunca se reemplaza ni se intenta tomar.

### Verificación post-deploy reproducible

Con el endpoint local explícito y las identidades públicas del manifiesto:

```console
$ ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 \
  TRUST_ESCROW_V3_EXPECTED_AUTHORITY=<authority-public-key> \
  TRUST_ESCROW_V3_ADVISOR_PUBKEY=<advisor-public-key> \
  TRUST_ESCROW_V3_TREASURY_PUBKEY=<treasury-public-key> \
  TRUST_ESCROW_V3_ARBITRATION_TREASURY_PUBKEY=<arbitration-treasury-public-key> \
  yarn verify:deploy
```

El verificador aborta antes de reportar éxito si difieren el endpoint, program
ID (Anchor.toml/IDL/cuenta on-chain), bytes del `.so` frente al `ProgramData`,
upgrade authority o cualquier campo de `Config` on-chain. Registra únicamente
public keys, hashes y endpoint; nunca lee ni imprime secretos de advisor.

### List runbooks available in this repository
```console
$ surfpool ls
Name                                    Description
deployment                              Deploy programs
```

### Start a Surfnet, automatically executing the `deployment` runbook on program recompile:
```console
$ surfpool start --watch
```

### Execute an existing runbook
```console
$ ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 surfpool run deployment
```

`pause_job` no detiene el timer de `Submitted`: solo es válido en `Created` o
`Funded` sin freelancer asignado. Para detener Surfpool/localnet use `Ctrl-C`.
