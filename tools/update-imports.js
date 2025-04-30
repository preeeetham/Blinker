const fs = require("fs");
const path = require("path");

const exts = new Set([".ts", ".tsx", ".js", ".jsx"]);
const root = process.cwd();

const replacements = [
  [/\/components\/ui\//g, "/ui-kit/primitives/"],
  [/\/components\/ui\/app-layout/g, "/ui-kit/app-layout"],
  [/\/components\/footer/g, "/ui-kit/footer"],
  [/\/components\/form\//g, "/features/forms/"],
  [/\/components\/DataCard\//g, "/features/data-card/"],
  [/\/components\/preview\//g, "/features/preview/"],
  [/\/components\/solana\//g, "/integrations/solana/"],
  [/\/components\/cluster\//g, "/integrations/cluster/"],
  [/\/hooks\//g, "/client-hooks/"],
  [/\/lib\//g, "/core/"],
  [/\/server\//g, "/backend/"],
  [/\/scripts\//g, "/ops/"],
  [/@\/components\/ui\//g, "@/ui-kit/primitives/"],
  [/@\/components\/ui\/app-layout/g, "@/ui-kit/app-layout"],
  [/@\/components\/footer/g, "@/ui-kit/footer"],
  [/@\/components\/form\//g, "@/features/forms/"],
  [/@\/components\/DataCard\//g, "@/features/data-card/"],
  [/@\/components\/preview\//g, "@/features/preview/"],
  [/@\/components\/solana\//g, "@/integrations/solana/"],
  [/@\/components\/cluster\//g, "@/integrations/cluster/"],
  [/@\/hooks\//g, "@/client-hooks/"],
  [/@\/lib\//g, "@/core/"],
  [/@\/server\//g, "@/backend/"],
  [/@\/scripts\//g, "@/ops/"]
];

function shouldSkipDir(name) {
  return name === "node_modules" || name === ".next" || name === ".git";
}

function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name.startsWith(".")) {
      if (shouldSkipDir(entry.name)) continue;
    }
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (shouldSkipDir(entry.name)) continue;
      walk(p);
    } else {
      const ext = path.extname(entry.name);
      if (!exts.has(ext)) continue;
      let source = fs.readFileSync(p, "utf8");
      let updated = source;
      for (const [re, repl] of replacements) {
        updated = updated.replace(re, repl);
      }
      if (updated !== source) {
        fs.writeFileSync(p, updated);
        console.log("updated", p);
      }
    }
  }
}

walk(root);
console.log("Import path updates complete.");
