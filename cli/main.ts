import { authenticate, encode_base64_url, register } from "passkey-client/passkey_demo_client.js";
import { Cookie, getSetCookies, setCookie } from "undici";

declare var fetch: typeof import("undici").fetch;

const host = "https://passkey-demo.localhost:8081";
const email = "test-user@example.com";

let sessionCookie: Cookie;
const response1 = await fetch(`${host}/users/register`, {
  method: "POST",
  body: email,
});
for (const cookie of getSetCookies(response1.headers)) {
  if (cookie.name === "passkey-demo") sessionCookie = cookie;
}

const ccr = (await (
  await fetch(`${host}/passkey/register/start`, {
    method: "POST",
    headers: {
      Cookie: `${sessionCookie.name}=${sessionCookie.value}`,
    },
  })
).json()) as any;

const created_public_key = await register(host, ccr);
const rpkc = {
  id: created_public_key.id,
  rawId: encode_base64_url(created_public_key.rawId),
  response: {
    attestationObject: encode_base64_url(created_public_key.response["attestationObject"]),
    clientDataJSON: encode_base64_url(created_public_key.response.clientDataJSON),
    transports: created_public_key.response["transports"],
  },
  type: created_public_key.type,
};

await fetch(`${host}/passkey/register/finish`, {
  method: "POST",
  body: JSON.stringify(rpkc),
  headers: {
    "Content-Type": "application/json",
    Cookie: `${sessionCookie.name}=${sessionCookie.value}`,
  },
});

await fetch(`${host}/logout`, {
  method: "POST",
  headers: {
    Cookie: `${sessionCookie.name}=${sessionCookie.value}`,
  },
});

const response2 = await fetch(`${host}/passkey/authenticate/start`, {
  method: "POST",
  body: email,
});
for (const cookie of getSetCookies(response2.headers)) {
  if (cookie.name === "passkey-demo") sessionCookie = cookie;
}

const rcr = await response2.json();
const public_key = await authenticate(host, rcr);
const pkc = {
  id: public_key.id,
  rawId: encode_base64_url(public_key.rawId),
  response: {
    clientDataJSON: encode_base64_url(public_key.response.clientDataJSON),
    authenticatorData: encode_base64_url(public_key.response["authenticatorData"]),
    signature: encode_base64_url(public_key.response["signature"]),
    userHandle: encode_base64_url(public_key.response["userHandle"]),
  },
  type: public_key.type,
};

await fetch(`${host}/passkey/authenticate/finish`, {
  method: "POST",
  body: JSON.stringify(pkc),
  headers: {
    "Content-Type": "application/json",
    Cookie: `${sessionCookie.name}=${sessionCookie.value}`,
  },
});
