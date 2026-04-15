import * as utils from "./utils.js"

async function getHashingMethods() {
  let response = await fetch("http://127.0.0.1:8080/app/hashing_methods")
  if (!response.ok) {
    throw new Error(`Response status: ${response.status}`)
  }
  return response.json()
}

async function displayHashingMethods() {
  let hashing_methods = getHashingMethods()

  let div = document.querySelectorAll("div.methods")

  let table = document.createElement("table")
  document.createElement("tr")

}
