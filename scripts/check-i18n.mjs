import { readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { API } from "typescript/unstable/sync";
import { SyntaxKind } from "typescript/unstable/ast";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const LOCALES_DIR = join(ROOT, "ui", "src", "locales");
const PROJECT = join(ROOT, "tsconfig.json");
const SOURCE_TAG = "en";
const PROBES = [
  0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 20, 21, 22, 25, 30, 40,
  50, 60, 70, 80, 90, 100, 101, 1000, 1000000,
];
const PLACEHOLDER = /\{[a-zA-Z]+\}/g;
const PLURAL_FORMS = ["zero", "one", "two", "few", "many", "other"];

function unwrapAssertion(node) {
  while (
    node.kind === SyntaxKind.AsExpression ||
    node.kind === SyntaxKind.SatisfiesExpression
  ) {
    node = node.expression;
  }
  return node;
}

function literalTexts(node) {
  const value = unwrapAssertion(node);
  if (value.kind === SyntaxKind.StringLiteral) return [value.text];
  if (
    value.kind === SyntaxKind.BinaryExpression &&
    value.operatorToken.kind === SyntaxKind.PlusToken
  ) {
    return [...literalTexts(value.left), ...literalTexts(value.right)];
  }
  return [];
}

function readMessage(property) {
  const value = unwrapAssertion(property.initializer);
  if (value.kind !== SyntaxKind.ObjectLiteralExpression) {
    return { plural: false, forms: {}, texts: literalTexts(property.initializer) };
  }
  const forms = {};
  for (const form of value.properties) {
    if (form.name?.text && form.initializer) {
      forms[form.name.text] = literalTexts(form.initializer);
    }
  }
  return { plural: true, forms, texts: Object.values(forms).flat() };
}

function readCatalog(sourceFile) {
  for (const statement of sourceFile.statements) {
    if (statement.kind !== SyntaxKind.VariableStatement) continue;
    for (const declaration of statement.declarationList.declarations) {
      if (!declaration.initializer) continue;
      const value = unwrapAssertion(declaration.initializer);
      if (value.kind !== SyntaxKind.ObjectLiteralExpression) continue;
      const messages = {};
      for (const property of value.properties) {
        if (!property.name?.text || !property.initializer) continue;
        messages[property.name.text] = readMessage(property);
      }
      return messages;
    }
  }
  return null;
}

function placeholders(texts) {
  const found = new Set();
  for (const text of texts) {
    for (const match of text.matchAll(PLACEHOLDER)) found.add(match[0]);
  }
  return found;
}

function missingFrom(expected, actual) {
  return [...expected].filter((token) => !actual.has(token));
}

function checkPlaceholders(file, key, message, reference, failures) {
  const expected = placeholders(reference.texts);
  const actual = placeholders(message.texts);
  const missing = missingFrom(expected, actual);
  const extra = missingFrom(actual, expected);
  if (missing.length > 0) {
    failures.push(`${file}: ${key}: missing placeholder ${missing.join(", ")}`);
  }
  if (extra.length > 0) {
    failures.push(`${file}: ${key}: unexpected placeholder ${extra.join(", ")}`);
  }
}

function checkPlural(file, key, tag, message, failures) {
  const forms = Object.keys(message.forms);
  for (const form of forms) {
    if (!PLURAL_FORMS.includes(form)) {
      failures.push(`${file}: ${key}: unknown plural form "${form}"`);
    }
  }
  if (!message.forms.other) {
    failures.push(`${file}: ${key}: plural missing "other"`);
  }
  const rules = new Intl.PluralRules(tag);
  const required = new Set(PROBES.map((n) => rules.select(n)));
  for (const category of required) {
    if (!message.forms[category]) {
      failures.push(`${file}: ${key}: plural missing "${category}" for ${tag}`);
    }
  }
}

function checkCatalog(file, tag, catalog, reference, failures) {
  for (const [key, message] of Object.entries(catalog)) {
    if (tag !== SOURCE_TAG && reference) {
      const expected = reference[key];
      if (!expected) {
        failures.push(`${file}: ${key}: missing in ${SOURCE_TAG}`);
      } else {
        checkPlaceholders(file, key, message, expected, failures);
      }
    }
    if (message.plural) {
      checkPlural(file, key, tag, message, failures);
    }
  }
  if (tag !== SOURCE_TAG && reference) {
    for (const key of Object.keys(reference)) {
      if (!(key in catalog)) failures.push(`${file}: ${key}: missing key`);
    }
  }
}

function loadCatalogs(files, failures) {
  const api = new API();
  const catalogs = {};
  try {
    const snapshot = api.updateSnapshot({ openProjects: [PROJECT] });
    const program = snapshot.getProjects()[0].program;
    for (const name of files) {
      const file = `ui/src/locales/${name}`;
      const sourceFile = program.getSourceFile(join(ROOT, file));
      if (!sourceFile) {
        failures.push(`${file}: not part of the TypeScript project`);
        continue;
      }
      const catalog = readCatalog(sourceFile);
      if (!catalog) {
        failures.push(`${file}: no catalog object literal found`);
        continue;
      }
      const tag = name.slice(0, -".ts".length).replace("_", "-");
      catalogs[tag] = { file, catalog };
    }
  } finally {
    api.close();
  }
  return catalogs;
}

function main() {
  const failures = [];
  const files = readdirSync(LOCALES_DIR)
    .filter((name) => name.endsWith(".ts") && name !== "index.ts")
    .sort();
  if (files.length === 0) {
    console.error("ui/src/locales: no locale catalogs found");
    return 1;
  }

  const catalogs = loadCatalogs(files, failures);
  const reference = catalogs[SOURCE_TAG]?.catalog;
  if (!reference) {
    failures.push(`ui/src/locales/${SOURCE_TAG}.ts: source catalog missing`);
  }
  for (const [tag, { file, catalog }] of Object.entries(catalogs)) {
    checkCatalog(file, tag, catalog, reference, failures);
  }

  if (failures.length > 0) {
    for (const failure of failures) console.error(failure);
    return 1;
  }
  console.log("i18n ok");
  return 0;
}

process.exit(main());
