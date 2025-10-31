const apiBase = window.location.origin;

const collectionSelect = document.getElementById("collectionSelect");
const refreshBtn = document.getElementById("refreshBtn");
const form = document.getElementById("docForm");
const notesContainer = document.getElementById("notes");

async function fetchCollections() {
  // You can manually define collections or dynamically list from backend if added
  const collections = ["notes", "tasks", "journal"];
  collectionSelect.innerHTML = "";
  for (const col of collections) {
    const opt = document.createElement("option");
    opt.value = col;
    opt.textContent = col;
    collectionSelect.appendChild(opt);
  }
}

async function loadNotes() {
  const collection = collectionSelect.value;
  const res = await fetch(`${apiBase}/get/${collection}`);
  const data = await res.json();

  notesContainer.innerHTML = "";
  if (!data.length) {
    notesContainer.innerHTML = "<p style='opacity:0.6'>No notes yet.</p>";
    return;
  }

  for (const note of data) {
    const el = document.createElement("div");
    el.className = "note";
    el.innerHTML = `
      <h3>${note.name}</h3>
      <p>${note.value}</p>
    `;
    notesContainer.appendChild(el);
  }
}

form.addEventListener("submit", async (e) => {
  e.preventDefault();
  const title = document.getElementById("title").value.trim();
  const content = document.getElementById("content").value.trim();
  const collection = collectionSelect.value;

  if (!title || !content) return;

  const res = await fetch(`${apiBase}/add/${collection}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name: title, value: content }),
  });

  if (res.ok) {
    form.reset();
    loadNotes();
  }
});

refreshBtn.addEventListener("click", loadNotes);

window.addEventListener("DOMContentLoaded", async () => {
  await fetchCollections();
  loadNotes();
});
