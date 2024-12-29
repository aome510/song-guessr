const makeApiRequest = async (url: string): Promise<Response> => {
  const response = await fetch(url);
  if (response.status !== 200) {
    const text = await response.text();
    throw new Error(`Failed to make API request ${url}: ${text}`);
  } else {
    return response;
  }
};

export { makeApiRequest };
