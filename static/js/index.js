"use strict";

(async () => {
  let inputEl = document.getElementById("text");
  let div = document.getElementById("qr");
  let buttonGen = document.getElementById("generate");
  let payload = "";
  inputEl.onchange = async (e) => {
    payload = e.target.value;
  };
  buttonGen.onclick = async () => {
    const data = await fetch("http://127.0.0.1:8080/api/svg", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      body: JSON.stringify({
        data: payload,
      }),
    })
      .then((response) => {
        return response.json();
      })
      .catch((error) => {
        console.log(error);
      });
    if (payload === "") {
      div.innerHTML = "Cannot Be Empty!";
      div.style.color = "red";
    } else {
      div.innerHTML = data.svg;
      div.style.color = "black";
      let btn = document.createElement("button");
      btn.className =
        "bg-clifford bg-blue-400 p-2 rounded-md m-2 hover:bg-blue-500 hover:text-white";
      btn.innerHTML = "Download";

      btn.onclick = async () => {
        const blob = new Blob([data.svg], { type: "image/svg+xml" });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = "image.svg";
        link.click();
      };

      div.appendChild(btn);
    }
  };
})();
