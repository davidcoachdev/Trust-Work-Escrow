import { readdirSync, readFileSync } from "node:fs";

export function validateDocumentation(source, state, jobs) {
  const required = [
    ["MAX_APPLICATIONS", "MAX_APPLICATIONS: usize = 50", "MAX_APPLICATIONS"],
    ["Submitted", "Submitted", "Submitted"],
    ["auto approval", "AUTO_APPROVAL_DELAY: i64 = 7 * 24 * 60 * 60", "604800"],
    ["evidence limit", "MAX_EVIDENCE_COUNT: u8 = 10", "10"],
    ["Evidence PDA", "pub struct Evidence", "Evidence PDA"],
    ["arbitration treasury", "arbitration_treasury", "arbitration_treasury"],
  ];
  for (const [label, code, doc] of required) {
    if (!source.includes(code) || !(state.includes(doc) || jobs.includes(doc))) {
      throw new Error(`Documentation drift: ${label}`);
    }
  }
  const allDocs = `${state}\n${jobs}`;
  if (/JobStatus[^`\n]*Received/.test(allDocs)) {
    throw new Error("Documentation must not describe a Received state");
  }
  if (/Dispute[^\n`]*evidence\s*:\s*Vec\s*<\s*Evidence\s*>/i.test(allDocs)) {
    throw new Error("Documentation drift: Evidence must be individual PDAs");
  }
  if (/(fee|5%).{0,80}(resolutor|resolver|asesor|\u00e1rbitro).{0,40}(recibe|cobra|se queda)/is.test(allDocs)) {
    throw new Error("Documentation drift: arbitration fee belongs to arbitration_treasury");
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const source = readFileSync("programs/trust-escrow-v3/src/lib.rs", "utf8");
  const state = readFileSync("docs/contract/03-estado.md", "utf8");
  const documentation = readdirSync("docs", { recursive: true })
    .filter((path) => path.endsWith(".md"))
    .map((path) => readFileSync(`docs/${path}`, "utf8"))
    .join("\n");
  validateDocumentation(source, state, documentation);
  console.log("documentation sync: ok");
}
