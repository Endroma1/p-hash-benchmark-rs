import * as utils from "./utils.js"

// Creates a list of checkboxes. The value is the index of the entry
function createCheckBoxList(container, list, name) {
  list.forEach((m, i) => {
    const item = document.createElement("div")

    const box = document.createElement("input")
    box.type = "checkbox"
    const box_id = `${m}Box`
    box.id = box_id
    box.name = name
    box.value = i
    item.appendChild(box)

    const label = document.createElement("label")
    label.htmlFor = box_id
    label.textContent = m

    item.appendChild(label)

    container.appendChild(item)
  })

}

async function displayModifications() {
  const div = document.querySelector(".modifications")

  const container = document.createElement("div")
  container.className = "div-list"

  const modifications = await utils.getModifications()

  createCheckBoxList(container, modifications, "modifications")

  div.appendChild(container)
}

async function displayHashingMethods() {
  const div = document.querySelector(".hashing_methods")

  const container = document.createElement("div")
  container.className = "div-list"

  const hashing_methods = await utils.getHashingMethods()

  createCheckBoxList(container, hashing_methods, "hashing_methods")

  div.appendChild(container)

}

async function handleSubmit() {
  const button = document.querySelector(".submit")
  button.onclick = async () => {
      const {modification_ids, hashing_method_ids} = handleCheckboxes()

      try{
    await startRun(hashing_method_ids,modification_ids)
      } catch (error) {
          console.log(error.message)
      }
  }
}

async function handleCheckboxes() {

    const modifications = document.querySelectorAll('.modifications input[type="checkbox"]:checked')
    const modification_ids = [...modifications].map((m) => m.value)

    const hashing_methods = document.querySelectorAll('.hashing_methods input[type="checkbox"]:checked')
    const hashing_method_ids = [...hashing_methods].map((m) => m.value)

    return {modification_ids: modification_ids, hashing_method_ids: hashing_method_ids}
}

// Takes two lists that contains method ids
async function startRun(hashing_methods, modifications){
    try{
        await fetch("http://localhost:8080/run/start", {
            method: "POST",
            body: JSON.stringify({hashing_methods, modifications})
        })
    } catch (error) {
        throw new Error(error.message)
    }
}


displayModifications()
displayHashingMethods()
handleSubmit()
utils.displayTopBar()
