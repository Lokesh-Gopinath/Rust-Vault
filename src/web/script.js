const API = `${window.location.origin}`;

async function init() {
  try {
    const r = await fetch(`${API}/collections`);
    if (!r.ok) throw new Error(r.status);
    const cols = await r.json();

    const sel = document.getElementById('collectionSelect');
    sel.innerHTML = cols.map(c => `<option>${c}</option>`).join('');
    sel.onchange = loadDocs;
    sel.onchange();          // load first collection
  } catch (e) {
    console.error(e);
    document.getElementById('collectionSelect').innerHTML =
      '<option disabled>Error loading collections</option>';
  }
}

async function loadDocs() {
  const col = document.getElementById('collectionSelect').value;
  const r = await fetch(`${API}/documents/${col}`);
  const docs = await r.json();
  const html = docs.map(d =>
    `<div class="note"><h3>${d.title}</h3><p>${d.content}</p></div>`).join('');
  document.getElementById('notes').innerHTML = html;
}

document.getElementById('docForm').onsubmit = async (e) => {
  e.preventDefault();
  const col = document.getElementById('collectionSelect').value;
  const title = document.getElementById('title').value;
  const content = document.getElementById('content').value;
  await fetch(`${API}/add/${col}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ title, content })
  });
  e.target.reset();
  loadDocs();
};

init();          // kick everything off
