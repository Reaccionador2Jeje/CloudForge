const ROUTES = new Map([
  ["/health/live", ["GET"]],
  ["/health/ready", ["GET"]],
  ["/v1/system/info", ["GET"]],
  ["/v1/system/components", ["GET"]],
  ["/v1/auth/bootstrap", ["POST"]],
  ["/v1/auth/login", ["POST"]],
  ["/v1/auth/me", ["GET"]],
  ["/v1/auth/logout", ["POST"]],
  ["/v1/instances", ["GET", "POST"]],
  ["/v1/nodes", ["GET"]],
  ["/v1/storage/quota", ["GET"]],
  ["/v1/storage/reservations", ["GET", "POST"]],
  ["/v1/storage/reservations/", ["DELETE"]],
  ["/v1/compute/providers", ["GET"]],
  ["/v1/compute/guest-profiles", ["GET"]],
  ["/v1/compute/installations", ["GET", "POST"]],
  ["/v1/images", ["GET", "POST"]],
]);

function jsonError(status, error) {
  return Response.json({ error }, {
    status,
    headers: { "Cache-Control": "no-store" },
  });
}

function isPrivateIpv4(host) {
  const parts = host.split(".");
  if (parts.length !== 4 || parts.some((part) => !/^\d{1,3}$/.test(part))) {
    return false;
  }

  const [a, b, c, d] = parts.map(Number);
  if ([a, b, c, d].some((part) => part > 255)) return true;

  return a === 10
    || a === 127
    || a === 0
    || (a === 169 && b === 254)
    || (a === 172 && b >= 16 && b <= 31)
    || (a === 192 && b === 168)
    || (a === 100 && b >= 64 && b <= 127);
}

function configuredOrigin(value) {
  if (!value) return null;

  try {
    const url = new URL(value);
    const host = url.hostname.toLowerCase();
    if (
      url.protocol !== "https:"
      || url.username
      || url.password
      || url.search
      || url.hash
      || (url.pathname !== "/" && url.pathname !== "")
      || host === "localhost"
      || host.endsWith(".localhost")
      || host.endsWith(".local")
      || host.endsWith(".internal")
      || host.endsWith(".lan")
      || isPrivateIpv4(host)
      || host.startsWith("[")
    ) {
      return null;
    }
    return url.origin;
  } catch {
    return null;
  }
}

async function proxyApi(request, env) {
  const incoming = new URL(request.url);
  const methods = ROUTES.get(incoming.pathname)
    || (incoming.pathname.startsWith("/v1/compute/installations/") ? ["GET"] : null)
    || (incoming.pathname.startsWith("/v1/storage/reservations/") && /^\/v1\/storage\/reservations\/[0-9a-f-]{36}$/i.test(incoming.pathname) ? ["DELETE"] : null);

  if (!methods) return jsonError(404, "API route not found");
  if (!methods.includes(request.method)) {
    return new Response(null, {
      status: 405,
      headers: { Allow: methods.join(", "), "Cache-Control": "no-store" },
    });
  }

  const origin = configuredOrigin(env.CLOUDFORGE_API_ORIGIN);
  const accessClientId = env.CF_ACCESS_CLIENT_ID;
  const accessClientSecret = env.CF_ACCESS_CLIENT_SECRET;

  // Fail closed: never proxy to an unprotected or unconfigured origin.
  if (!origin || !accessClientId || !accessClientSecret) {
    return jsonError(503, "CloudForge API backend is not configured or protected");
  }

  const upstreamUrl = new URL(incoming.pathname + incoming.search, origin);
  const headers = new Headers();
  for (const name of ["accept", "content-type", "authorization"]) {
    const value = request.headers.get(name);
    if (value) headers.set(name, value);
  }
  headers.set("CF-Access-Client-Id", accessClientId);
  headers.set("CF-Access-Client-Secret", accessClientSecret);
  headers.set("Cache-Control", "no-store");

  try {
    const upstream = await fetch(new Request(upstreamUrl, {
      method: request.method,
      headers,
      body: request.method === "GET" || request.method === "HEAD" ? undefined : request.body,
      redirect: "manual",
    }));

    if (upstream.status >= 300 && upstream.status < 400) {
      return jsonError(502, "Unexpected redirect from CloudForge API origin");
    }

    const responseHeaders = new Headers(upstream.headers);
    responseHeaders.delete("set-cookie");
    responseHeaders.delete("access-control-allow-origin");
    responseHeaders.set("Cache-Control", "no-store");
    return new Response(upstream.body, {
      status: upstream.status,
      headers: responseHeaders,
    });
  } catch {
    return jsonError(502, "CloudForge API origin is unavailable");
  }
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    if (url.pathname.startsWith("/health/") || url.pathname.startsWith("/v1/")) {
      return proxyApi(request, env);
    }
    return env.ASSETS.fetch(request);
  },
};
