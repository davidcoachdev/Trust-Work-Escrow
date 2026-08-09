import test from "node:test";
import assert from "node:assert/strict";
import { validateDocumentation } from "./check-doc-sync.mjs";

test("detecta el drift semántico de Evidence PDA y treasury de arbitraje", () => {
  const source = `const MAX_APPLICATIONS: usize = 50; const AUTO_APPROVAL_DELAY: i64 = 7 * 24 * 60 * 60; const MAX_EVIDENCE_COUNT: u8 = 10; Submitted`;
  assert.throws(
    () => validateDocumentation(source, "MAX_APPLICATIONS Submitted 604800 10 Evidence PDA arbitration_treasury Dispute.evidence: Vec<Evidence>", ""),
    /Evidence PDA/,
  );
});

test("acepta el contrato documental vigente", () => {
  const source = `const MAX_APPLICATIONS: usize = 50; const AUTO_APPROVAL_DELAY: i64 = 7 * 24 * 60 * 60; const MAX_EVIDENCE_COUNT: u8 = 10; pub struct Evidence {} Submitted arbitration_treasury`;
  const docs = "Evidence PDA individual; arbitration_treasury; Submitted; 604800; MAX_APPLICATIONS; 10";
  assert.doesNotThrow(() => validateDocumentation(source, docs, docs));
});
