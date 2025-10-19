const API = "https://rust-vault.onrender.com";

const collectionSelect = document.getElementById("collectionSelect");
const docForm = document.getElementById("docForm");

async function loadDocuments() {
  const collection = collectionSelect.value;
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
loadDocuments();
