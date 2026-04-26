import * as utils from "./utils.js"

const MODIFICATIONS_LINK = "./modifications.html"
const HASHING_METHODS_LINK = "./hashing_methods.html"


async function displayRuns() {
  const div = document.querySelector("div.runs")
  try {
    const result = await utils.getRuns()

    const table = document.createElement("table")

    const id_bold = document.createElement("b")
    id_bold.textContent = "id"

    const date_bold = document.createElement("b")
    date_bold.textContent = "date"

    utils.addRow(table, id_bold, date_bold)

    result.runs.map((run) => {
      const date = new Date(run.timestamp).toLocaleDateString();
      utils.addRow(table, run.id, date)
    })

    div.appendChild(table)
  } catch (error) {
    div.textContent = error.message
  }
}


async function displayApp() {
  const div = document.querySelector("div.app")
  try {
    const table = document.createElement("table")

    let app_info = await utils.getAppInfo()

    const field_bold = document.createElement("b")
    field_bold.textContent = "field"

    const value_bold = document.createElement("b")
    value_bold.textContent = "value"

    utils.addRow(table, field_bold, value_bold)

    utils.addRow(table, "status", app_info.status)

    if (app_info.status === "initialized") {
      const hash_link = document.createElement("a")
      hash_link.setAttribute("href", HASHING_METHODS_LINK)
      hash_link.textContent = "hashing_methods"

      const mod_link = document.createElement("a")
      mod_link.setAttribute("href", MODIFICATIONS_LINK)
      mod_link.textContent = "modifications"

      utils.addRow(table, hash_link, app_info.data.hashing_methods.length)
      utils.addRow(table, mod_link, app_info.data.modifications.length)
    }

    div.appendChild(table)

  } catch (error) {
    div.textContent = error.message
  }
}

async function AppInit() {
  const button = document.querySelector(".app-init")
  button.onclick = async () => {
    try {
      await utils.appInit()
    } catch (error) {
      console.log(error.message)
      // Show error to user when implemented
    }
    location.reload() // Reload app status

  }
}

displayRuns()
displayApp()
AppInit()
utils.displayTopBar()
