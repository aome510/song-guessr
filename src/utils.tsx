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

export { get, post };
