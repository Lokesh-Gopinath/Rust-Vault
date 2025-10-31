const API = "https://rust-vault.onrender.com"; // your Render backend

const collectionSelect = document.getElementById("collectionSelect");
const docForm = document.getElementById("docForm");
const notesContainer = document.getElementById("notes");
const refreshBtn = document.getElementById("refreshBtn");

async function loadCollections() {
  try {
    const res = await fetch(`${API}/collections`);
    const collections = await res.json();

    collectionSelect.innerHTML = "";
    collections.forEach(name => {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      collectionSelect.appendChild(option);
    });

    if (collections.length > 0) loadDocuments();
  } catch (err) {
    console.error("Failed to load collections", err);
  }
}

async function loadDocuments() {
  const collection = collectionSelect.value;
  if (!collection) return;

  notesContainer.innerHTML = "<p>Loading...</p>";
  try {
    const res = await fetch(`${API}/documents/${collection}`);
    const data = await res.json();

    notesContainer.innerHTML = "";
    if (data.length === 0) {
      notesContainer.innerHTML = "<p>No notes found.</p>";
      return;
    }

    data.forEach(doc => {
      const div = document.createElement("div");
      div.className = "note";
      div.innerHTML = `<h3>${doc.title}</h3><p>${doc.content}</p>`;
      notesContainer.appendChild(div);
    });
  } catch (err) {
    notesContainer.innerHTML = "<p>Error loading notes.</p>";
  }
}

docForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const collection = collectionSelect.value;
  const title = document.getElementById("title").value;
  const content = document.getElementById("content").value;

  if (!collection) return alert("Select a collection first!");

  await fetch(`${API}/add/${collection}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title, content }),
  });

  docForm.reset();
  loadDocuments();
});

collectionSelect.addEventListener("change", loadDocuments);
refreshBtn.addEventListener("click", loadCollections);

// Initialize
loadCollections();
