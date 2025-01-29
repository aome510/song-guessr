import { User } from "./model";

async function get(url: string): Promise<Response> {
  const response = await fetch(url);
  if (response.status !== 200) {
    const text = await response.text();
    throw new Error(`Failed to send GET request ${url}: ${text}`);
  } else {
    return response;
  }
}

async function post<T>(url: string, body: T): Promise<Response> {
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  if (response.status !== 200) {
    const text = await response.text();
    throw new Error(`Failed to send POST request ${url}: ${text}`);
  } else {
    return response;
  }
}

function getUserData(): User | null {
  const id = localStorage.getItem("userId");
  if (id === null) {
    return null;
  }
  const name = localStorage.getItem("userName");
  if (name === null) {
    return null;
  }
  return { id, name };
}

function getClientId(): string | null {
  return localStorage.getItem("clientId");
}

export { get, post, getUserData, getClientId };
