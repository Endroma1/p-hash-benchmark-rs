
export async function getRuns() {
  const response = await fetch("http://localhost:8080/runs");
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`)
  }
  const result = await response.json();
  return result
}

export function addRows(table, rows_map) {
  rows_map.forEach((val, key) => {
    addRow(table, val, key)
  })
}

export function addRow(table, cell_1, cell_2) {
  const row = document.createElement("tr");

  [cell_1, cell_2].forEach((c) => {

    const td = document.createElement("td")

    if (c instanceof HTMLElement) {
      td.appendChild(c)
    } else {
      td.textContent = c
    }
    row.appendChild(td)
  })
  table.appendChild(row)
}

export async function getAppInfo() {
  const response = await fetch("http://127.0.0.1:8080/app")
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`)
  }
  return await response.json()
}

export async function appInit() {
  const response = await fetch("http://127.0.0.1:8080/app/init")
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`)
  }
}

export async function getHashingMethods() {
  const response = await fetch("http://127.0.0.1:8080/hashing_methods")
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`)
  }
  return await response.json()
}
export async function getModifications() {
  const response = await fetch("http://127.0.0.1:8080/modifications")
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`)
  }
  return await response.json()
}

export function displayTopBar() {
  let body = document.querySelector("body")

  let top_bar = document.createElement("div")
  top_bar.className = "top_bar"
  body.prepend(top_bar)

  let title = document.createElement("div")
  title.className = "title"
  top_bar.appendChild(title)

  let header = document.createElement("h1")
  header.textContent = "PHash"
  title.appendChild(header)
}
