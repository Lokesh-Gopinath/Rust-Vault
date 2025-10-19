const API = `${window.location.origin}`;

const $     = q => document.querySelector(q);
const $sel  = $('#collectionSelect');
const $form = $('#docForm');
const $notes= $('#notes');
const $toast= $('#toast');

function toast(msg) {
  $toast.textContent = msg;
  $toast.classList.add('show');
  setTimeout(() => $toast.classList.remove('show'), 2000);
}

async function fetchJSON(path) {
  const res = await fetch(`${API}${path}`);
  if (!res.ok) throw new Error(res.statusText);
  return res.json();
}

async function loadCollections() {
  try {
    const cols = await fetchJSON('/collections');
    $sel.innerHTML = cols.map(c => `<option value="${c}">${c}</option>`).join('');
    loadDocuments();
  } catch {
    toast('Could not load collections');
    $sel.innerHTML = '<option disabled>Error</option>';
  }
}

async function loadDocuments() {
  const col = $sel.value;
  if (!col) return;
  try {
    const docs = await fetchJSON(`/documents/${col}`);
    $notes.innerHTML = docs.map(d => `
      <div class="note">
        <button class="del" data-id="${d._id.$oid}">&times;</button>
        <h3>${d.title}</h3>
        <p>${d.content}</p>
      </div>`).join('');
    $$notes('.del').forEach(btn =>
      btn.onclick = async e => {
        e.stopPropagation();
        await fetch(`${API}/delete/${col}/${btn.dataset.id}`, { method: 'DELETE' });
        toast('Deleted');
        loadDocuments();
      });
  } catch {
    toast('Could not load documents');
  }
}

$form.addEventListener('submit', async e => {
  e.preventDefault();
  const col = $sel.value;
  const title   = $('#title').value.trim();
  const content = $('#content').value.trim();
  if (!title || !content) return;
  try {
    await fetch(`${API}/add/${col}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title, content })
    });
    $form.reset();
    toast('Added');
    loadDocuments();
  } catch {
    toast('Add failed');
  }
});

$('#refreshBtn').onclick = () => loadDocuments();
const $$notes = sel => $notes.querySelectorAll(sel);

loadCollections();
