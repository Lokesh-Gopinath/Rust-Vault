const API = "https://rust-vault.onrender.com"; // replace with your Render URL

const collectionSelect = document.getElementById("collectionSelect");
const docForm = document.getElementById("docForm");

async function loadCollections() {
  const res = await fetch(`${API}/collections`);
  const collections = await res.json();

  collectionSelect.innerHTML = "";
  collections.forEach(name => {
    const option = document.createElement("option");
    option.value = name;
    option.textContent = name;
    collectionSelect.appendChild(option);
  });

  // Load documents for first collection by default
  if (collections.length > 0) loadDocuments();
}

async function loadDocuments() {
  const collection = collectionSelect.value;
  if (!collection) return;

  const res = await fetch(`${API}/documents/${collection}`);
  const data = await res.json();
  const container = document.getElementById("notes");
  container.innerHTML = "";
  data.forEach(doc => {
    const div = document.createElement("div");
    div.className = "note";
    div.innerHTML = `<h3>${doc.title}</h3><p>${doc.content}</p>`;
    container.appendChild(div);
  });
}

// Load documents when collection changes
collectionSelect.addEventListener("change", loadDocuments);

// Handle form submit
docForm.addEventListener("submit", async (e) => {
  e.preventDefault();
  const collection = collectionSelect.value;
  const title = document.getElementById("title").value;
  const content = document.getElementById("content").value;

  await fetch(`${API}/add/${collection}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title, content }),
  });

  docForm.reset();
  loadDocuments();
});

// Initial load
loadCollections();
