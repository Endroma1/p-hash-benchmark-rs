import * as utils from "./utils.js"

function handleImages(username){
    const formdata = new FormData()

    formdata.append("username", username)

    const files = document.getElementById("images").files

    for (const file of files) {
        formdata.append("images", file)
    }

    return formdata
}


function handleEntryName(){
    return document.getElementById("name").value
}

// Sends images and entry name to application
async function postImages(images){
    await fetch("http://localhost:8080/images/submit", {method: "POST", body: images})
}

async function handleSubmit(){
    let submit_button = document.querySelector("button.submit")
    submit_button.onclick = async (_) => {
        try{
            const name = handleEntryName()
            const formdata = handleImages(name)
            await postImages(formdata)
        } catch(error) {
            console.log(error.message)
        }
    }

}

utils.displayTopBar()
handleSubmit()
