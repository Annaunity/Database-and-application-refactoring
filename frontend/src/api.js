const BASE_URL = '/api/v1';

async function doPost(url, body) {
  const headers = new Map();
  headers.set('Content-Type', 'application/json');

  const token = window.localStorage.getItem("token");
  if (token != null) {
    headers.set('Authorization', token);
  }

  const response = await fetch(BASE_URL + url, {
    method: 'POST',
    headers,
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

async function doGet(url) {
  const headers = new Map();
  headers.set('Content-Type', 'application/json');

  const token = window.localStorage.getItem("token");
  if (token != null) {
    headers.set('Authorization', token);
  }

  const response = await fetch(BASE_URL + url, { headers });

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

export async function getUser(username) {
  return doGet(`/user/${username}`);
}

export async function getCurrentUser() {
  return doGet(`/user/me`);
}

export async function auth(data) {
  return doPost('/auth', data);
}

export async function createDrawing(data) {
  return doPost(`/drawing`, data);
}

export async function getOwnedDrawings() {
  return doGet(`/drawing/owned`);
}
