#!/bin/bash

# Trust Work Escrow v2 - Demo Script
# For Hackathon - March 23, 2026

set -e

CLI="./target/debug/trust-escrow"
BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BOLD}🚀 Trust Work Escrow v2 - Hackathon Demo${NC}"
echo "=========================================="
echo ""

echo -e "${BOLD}1. Program Deployment${NC}"
echo "Program ID: 28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA"
echo "Devnet Explorer: https://explorer.solana.com/address/28QTH6qfG2iKDVXNY8nUTKDjx8yrBrQpnvXCPyJsrwuA?cluster=devnet"
echo ""

echo -e "${BOLD}2. CLI Status Check${NC}"
$CLI status
echo ""

echo -e "${BOLD}3. Wallet Balance${NC}"
$CLI payment balance
echo ""

echo -e "${BOLD}4. Configuration${NC}"
$CLI config show
echo ""

echo -e "${BOLD}5. Available Commands${NC}"
echo "User Management: trust-escrow user create --name \"Name\""
echo "Job Management:  trust-escrow job create --title \"Title\" --description \"Desc\" --amount 1.0"
echo "Payments:        trust-escrow payment balance"
echo "Disputes:        trust-escrow dispute list"
echo "Config:          trust-escrow config set key value"
echo ""

echo -e "${BOLD}6. SDK Operations (51 total)${NC}"
echo "See trust-escrow-sdk documentation for all operations"
echo ""

echo -e "${GREEN}✅ Demo Complete!${NC}"
echo ""
echo -e "${YELLOW}Note: Some SDK operations are still being implemented.${NC}"
echo -e "${YELLOW}The smart contract is fully deployed and functional.${NC}"
echo -e "${YELLOW}CLI connects successfully to devnet program.${NC}"