// Rustok RPC proxy — single stable TLS endpoint for Ethereum RPCs and Blockscout explorers.
// Why: Android rustls-platform-verifier (0.6.x) enforces strict OCSP revocation,
// which fails against Let's Encrypt certs (OCSP EOL August 2025). Proxying through
// Cloudflare gives one predictable cert with OCSP URL that mobile clients accept.

type ChainName =
  | "ethereum"
  | "arbitrum"
  | "base"
  | "optimism"
  | "zksync"
  | "sepolia";

const RPC_UPSTREAM: Record<ChainName, string[]> = {
  ethereum: [
    "https://ethereum-rpc.publicnode.com",
    "https://cloudflare-eth.com",
    "https://eth.drpc.org",
  ],
  arbitrum: [
    "https://arbitrum-one-rpc.publicnode.com",
    "https://arbitrum.drpc.org",
  ],
  base: ["https://base-rpc.publicnode.com", "https://base.drpc.org"],
  optimism: [
    "https://optimism-rpc.publicnode.com",
    "https://optimism.drpc.org",
  ],
  zksync: [
    "https://mainnet.era.zksync.io",
    "https://zksync.drpc.org",
  ],
  sepolia: [
    "https://ethereum-sepolia-rpc.publicnode.com",
    "https://sepolia.drpc.org",
  ],
};

const EXPLORER_UPSTREAM: Partial<Record<ChainName, string>> = {
  ethereum: "https://eth.blockscout.com",
  arbitrum: "https://arbitrum.blockscout.com",
  base: "https://base.blockscout.com",
  optimism: "https://optimism.blockscout.com",
  sepolia: "https://eth-sepolia.blockscout.com",
};

const CORS_HEADERS: Record<string, string> = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, X-Rustok-Client",
  "Access-Control-Max-Age": "86400",
};

function isChain(s: string): s is ChainName {
  return s in RPC_UPSTREAM;
}

async function proxyRpc(chain: ChainName, req: Request): Promise<Response> {
  const body = await req.arrayBuffer();
  const upstreams = RPC_UPSTREAM[chain];
  let lastErr = "no upstream";

  for (const upstream of upstreams) {
    try {
      const resp = await fetch(upstream, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body,
      });
      if (resp.ok) {
        return new Response(resp.body, {
          status: resp.status,
          headers: {
            "Content-Type":
              resp.headers.get("Content-Type") ?? "application/json",
            "X-Upstream": upstream,
            ...CORS_HEADERS,
          },
        });
      }
      lastErr = `${upstream} -> ${resp.status}`;
    } catch (e) {
      lastErr = `${upstream} -> ${e instanceof Error ? e.message : String(e)}`;
    }
  }

  return new Response(
    JSON.stringify({ error: "all upstreams failed", detail: lastErr }),
    { status: 502, headers: { "Content-Type": "application/json", ...CORS_HEADERS } },
  );
}

async function proxyExplorer(
  chain: ChainName,
  rest: string,
  req: Request,
): Promise<Response> {
  const upstream = EXPLORER_UPSTREAM[chain];
  if (!upstream) {
    return new Response(JSON.stringify({ error: "explorer not configured" }), {
      status: 404,
      headers: { "Content-Type": "application/json", ...CORS_HEADERS },
    });
  }

  const url = new URL(req.url);
  const target = `${upstream}/${rest}${url.search}`;

  const resp = await fetch(target, {
    method: req.method,
    headers: {
      Accept: req.headers.get("Accept") ?? "application/json",
    },
  });

  return new Response(resp.body, {
    status: resp.status,
    headers: {
      "Content-Type":
        resp.headers.get("Content-Type") ?? "application/json",
      "X-Upstream": upstream,
      ...CORS_HEADERS,
    },
  });
}

export default {
  async fetch(req: Request): Promise<Response> {
    if (req.method === "OPTIONS") {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    const url = new URL(req.url);
    const parts = url.pathname.split("/").filter(Boolean);

    // /rpc/{chain}
    if (parts[0] === "rpc" && parts.length === 2 && req.method === "POST") {
      const chain = parts[1] ?? "";
      if (!isChain(chain)) {
        return new Response(JSON.stringify({ error: `unknown chain: ${chain}` }), {
          status: 404,
          headers: { "Content-Type": "application/json", ...CORS_HEADERS },
        });
      }
      return proxyRpc(chain, req);
    }

    // /explorer/{chain}/...path
    if (parts[0] === "explorer" && parts.length >= 2) {
      const chain = parts[1] ?? "";
      if (!isChain(chain)) {
        return new Response(JSON.stringify({ error: `unknown chain: ${chain}` }), {
          status: 404,
          headers: { "Content-Type": "application/json", ...CORS_HEADERS },
        });
      }
      const rest = parts.slice(2).join("/");
      return proxyExplorer(chain, rest, req);
    }

    // /health
    if (parts[0] === "health") {
      return new Response(
        JSON.stringify({ ok: true, service: "rustok-rpc-proxy" }),
        { headers: { "Content-Type": "application/json", ...CORS_HEADERS } },
      );
    }

    return new Response(
      JSON.stringify({
        service: "rustok-rpc-proxy",
        usage: {
          rpc: "POST /rpc/{ethereum|arbitrum|base|optimism|zksync|sepolia}",
          explorer: "GET /explorer/{chain}/api?module=...",
          health: "GET /health",
        },
      }),
      { headers: { "Content-Type": "application/json", ...CORS_HEADERS } },
    );
  },
};
