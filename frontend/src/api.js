const BASE_URL = '/api/v1';

async function doPost(url, body) {
  const response = await fetch(BASE_URL + url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(body)
  });

  let data = null;

  try {
    data = await response.json();
  } catch (e) {}

  if (!response.ok) {
    console.error(`Error in ${url}: ${data.message}`);
    throw new Error(data.message);
  }

  return data;
}

export async function createUser(data) {
  return doPost('/user', data);
}

export async function auth(data) {
  return doPost('/auth', data);
}
