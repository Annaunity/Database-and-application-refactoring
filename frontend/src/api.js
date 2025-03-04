const BASE_URL = '/api/v1';

async function doRequest(method, url, body, blob) {
  const headers = new Map();

  if (body) {
    headers.set('Content-Type', 'application/json');
  }

  const token = window.localStorage.getItem("token");
  if (token != null) {
    headers.set('Authorization', token);
  }

  let actualBody = null;

  if (body) actualBody = JSON.stringify(body);

  if (blob) {
    actualBody = new FormData();
    actualBody.append("file", blob);
  }

  const response = await fetch(BASE_URL + url, {
    method,
    headers,
    body: actualBody
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

async function doRequestBlob(method, url, body) {
  const headers = new Map();
  headers.set('Content-Type', 'application/json');

  const token = window.localStorage.getItem("token");
  if (token != null) {
    headers.set('Authorization', token);
  }

  const response = await fetch(BASE_URL + url, {
    method,
    headers,
    body: body && JSON.stringify(body)
  });

  let data = null;

  try {
    data = await response.blob();
  } catch (e) {}

  if (!response.ok) {
    data = JSON.parse(await data.text());
    console.error(`Error in ${url}: ${data.message}`);
    throw new Error(data.message);
  }

  return data;
}

async function doPost(url, body) {
  return doRequest('POST', url, body)
}

async function doGet(url, body) {
  return doRequest('GET', url, body)
}

async function doDelete(url, body) {
  return doRequest('DELETE', url, body)
}

async function doPatch(url, body) {
  return doRequest('PATCH', url, body)
}

export async function createUser(data) {
  return doPost('/user', data);
}

export async function getUser(username) {
  return doGet(`/user/${username}`);
}

export async function updateUser(username, data) {
  return doPatch(`/user/${username}`, data);
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

export async function getDrawing(id) {
  return doGet(`/drawing/${id}`);
}

export async function getDrawingVersions(id) {
  return doGet(`/drawing/${id}/version`);
}

export async function deleteDrawing(id) {
  return doDelete(`/drawing/${id}`);
}

export async function updateDrawing(id, data) {
  return doPatch(`/drawing/${id}`, data);
}

export async function blurDrawing(id) {
  return doPost(`/drawing/${id}/operation/blur`);
}

export async function invertDrawing(id) {
  return doPost(`/drawing/${id}/operation/invert`);
}

export async function uploadDrawingNewVersion(id, blob) {
  return doRequest('PUT', `/drawing/${id}/version/latest`, null, blob);
}

export async function getDrawingLatestVersion(id) {
  return doRequestBlob('GET', `/drawing/${id}/version/latest`);
}

export async function getDrawingLatestVersionThumbnail(id) {
  return doRequestBlob('GET', `/drawing/${id}/version/latest?thumbnail=true`);
}

export async function getDrawingVersion(id, ver) {
  return doRequestBlob('GET', `/drawing/${id}/version/${ver}`);
}

export async function getDrawingVersionThumbnail(id, ver) {
  return doRequestBlob('GET', `/drawing/${id}/version/${ver}?thumbnail=true`);
}

export async function getSessions() {
  return doGet(`/auth/session`);
}

export async function endCurrentSession() {
  return doDelete(`/auth`);
}

export async function endSession(tokenId) {
  let params = new URLSearchParams({ tokenId })
  return doDelete(`/auth/session?${params.toString()}`);
}

