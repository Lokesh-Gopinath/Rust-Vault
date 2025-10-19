const API = "https://YOUR-RENDER-URL.onrender.com/api/notes"; // change later

async function loadNotes() {
  const res = await fetch(API);
  const notes = await res.json();
  const container = document.getElementById("notes");
  container.innerHTML = "";
  notes.forEach(n => {
    const div = document.createElement("div");
    div.className = "note";
    div.innerHTML = `<h3>${n.title}</h3><p>${n.content}</p>`;
    container.appendChild(div);
  });
}

document.getElementById("noteForm").addEventListener("submit", async (e) => {
  e.preventDefault();
  const title = document.getElementById("title").value;
  const content = document.getElementById("content").value;

  await fetch(API, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title, content }),
  });

  document.getElementById("noteForm").reset();
  loadNotes();
});

loadNotes();
