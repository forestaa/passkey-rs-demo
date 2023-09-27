import { encode_base64_url, register  } from "passkey-client/passkey_demo_client.js";
import { Cookie, getSetCookies, setCookie } from "undici";

declare var fetch: typeof import('undici').fetch;

const host = "https://passkey-demo.localhost:8081";

let authCookie: Cookie;
const response = await fetch(`${host}/users/register`, { method: 'POST', body: 'test-user@example.com' });
for (const cookie of getSetCookies(response.headers)) {
  if (cookie.name === 'passkey-demo') authCookie = cookie;
}

const ccr = await (await fetch(`${host}/passkey/register/start`, {
  method: 'POST',
  headers: {
    'Cookie': `${authCookie.name}=${authCookie.value}`,
  },
})).json() as any;

const public_key = await register(host, ccr);
const rpkc = {
  id: public_key.id,
  rawId: encode_base64_url(public_key.rawId),
  response: {
    attestationObject: encode_base64_url(public_key.response['attestationObject']),
    clientDataJSON: encode_base64_url(public_key.response.clientDataJSON),
    transports: public_key.response['transports'],
  },
  type: public_key.type,
};

await fetch(`${host}/passkey/register/finish`, {
  method: 'POST',
  body: JSON.stringify(rpkc),
  headers: {
    'Content-Type': 'application/json',
    'Cookie': `${authCookie.name}=${authCookie.value}`,
  },
});
