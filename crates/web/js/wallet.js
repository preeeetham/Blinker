// Wallet Standard bridge — the only JavaScript in Blinker.
//
// Browser wallets (Phantom, Solflare, Backpack, …) expose themselves to pages
// through the Wallet Standard event protocol, which is a JS API: there is no
// way to reach a wallet extension from WASM without a few lines of glue. This
// file is that glue and nothing more; all logic (building, checking and
// submitting transactions) lives in Rust.

const wallets = new Map();
let current = null;
let account = null;
let started = false;

function register(...list) {
  for (const w of list) {
    if (!w || !w.features || !w.features["standard:connect"]) continue;
    if (!Array.isArray(w.chains) || !w.chains.some((c) => String(c).startsWith("solana:"))) continue;
    wallets.set(w.name, w);
  }
  return () => {};
}

function start() {
  if (started || typeof window === "undefined") return;
  started = true;
  const api = Object.freeze({ register });
  window.addEventListener("wallet-standard:register-wallet", (e) => {
    try { e.detail(api); } catch (_) {}
  });
  try { window.dispatchEvent(new CustomEvent("wallet-standard:app-ready", { detail: api })); } catch (_) {}
}

export function list_wallets() {
  start();
  return JSON.stringify([...wallets.values()].map((w) => ({ name: w.name, icon: w.icon || "" })));
}

export function remembered() {
  try { return localStorage.getItem("blinker.wallet") || ""; } catch (_) { return ""; }
}

export async function connect(name, silent) {
  start();
  const w = wallets.get(name);
  if (!w) throw new Error(`${name} is not installed`);
  const res = await w.features["standard:connect"].connect(silent ? { silent: true } : undefined);
  const acct = ((res && res.accounts) || w.accounts || [])[0];
  if (!acct) throw new Error("No account was authorised");
  current = w;
  account = acct;
  try { localStorage.setItem("blinker.wallet", name); } catch (_) {}
  return acct.address;
}

export async function disconnect() {
  try { await current?.features["standard:disconnect"]?.disconnect(); } catch (_) {}
  current = null;
  account = null;
  try { localStorage.removeItem("blinker.wallet"); } catch (_) {}
}

const fromB64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
const toB64 = (bytes) => btoa(Array.from(bytes, (b) => String.fromCharCode(b)).join(""));

// Prefers signTransaction (wallet signs, Blinker submits to its own cluster);
// falls back to signAndSendTransaction.
export async function sign(txB64, chain) {
  if (!current || !account) throw new Error("Connect a wallet first");
  const transaction = fromB64(txB64);
  const signOnly = current.features["solana:signTransaction"];
  if (signOnly) {
    const [out] = await signOnly.signTransaction({ account, transaction, chain });
    return JSON.stringify({ mode: "signed", tx: toB64(out.signedTransaction) });
  }
  const signSend = current.features["solana:signAndSendTransaction"];
  if (signSend) {
    const [out] = await signSend.signAndSendTransaction({ account, transaction, chain });
    return JSON.stringify({ mode: "sent", signature: toB64(out.signature) });
  }
  throw new Error("This wallet can't sign Solana transactions");
}
