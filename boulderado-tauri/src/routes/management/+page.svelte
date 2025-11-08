<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type Product = {
    id: number;
    name: string;
    price_cents: number;
    accent?: string | null;
    icon?: string | null;
    note?: string | null;
  };

  type Transaction = {
    id: number;
    product_id?: number | null;
    quantity: number;
    total_cents: number;
    description?: string | null;
    created_at: string;
  };

  let products: Product[] = [];
  let transactions: Transaction[] = [];
  let loading = true;
  let errorMsg = "";

  onMount(async () => {
    try {
      const [productData, transactionData] = await Promise.all([
        invoke<Product[]>("list_products"),
        invoke<Transaction[]>("list_transactions")
      ]);
      products = productData ?? [];
      transactions = transactionData ?? [];
    } catch (error) {
      errorMsg = "Daten konnten nicht geladen werden.";
      console.error(error);
    } finally {
      loading = false;
    }
  });

  function formatPrice(cents: number) {
    return new Intl.NumberFormat("de-DE", {
      style: "currency",
      currency: "EUR"
    }).format((cents ?? 0) / 100);
  }
</script>

<main class="management-shell">
  <header>
    <h1>Verwaltung</h1>
    <p>Verwalten Sie Produktinventar, Kundendaten und zukünftige Datenerfassungen.</p>
  </header>

  {#if loading}
    <p class="status">Lade Daten...</p>
  {:else if errorMsg}
    <p class="status error">{errorMsg}</p>
  {/if}

  <section class="panel">
    <div class="panel-head">
      <div>
        <h2>Produkte</h2>
        <p>Aktuelle Artikel aus der SQLite-Datenbank.</p>
      </div>
      <button disabled title="Wird später implementiert">Neues Produkt</button>
    </div>
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Preis</th>
            <th>Akzent</th>
            <th>Icon</th>
            <th>Notiz</th>
          </tr>
        </thead>
        <tbody>
          {#if products.length === 0}
            <tr>
              <td colspan="5">Keine Produkte vorhanden.</td>
            </tr>
          {:else}
            {#each products as product}
              <tr>
                <td>{product.name}</td>
                <td>{formatPrice(product.price_cents)}</td>
                <td>
                  {#if product.accent}
                    <span class="swatch" style={`--accent:${product.accent}`}></span>
                    {product.accent}
                  {:else}
                    -
                  {/if}
                </td>
                <td>{product.icon ?? "-"}</td>
                <td>{product.note ?? "-"}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </section>

  <section class="panel">
    <div class="panel-head">
      <div>
        <h2>Kunden & Mitgliedschaften</h2>
        <p>Platzhalter für Kundendaten und Statusverwaltung.</p>
      </div>
      <button disabled title="Funktion folgt">Kunden importieren</button>
    </div>
    <div class="placeholder">
      <p>
        Dieses Modul wird Stammdaten, Mitgliedschaften, Laufzeiten und Zahlungsübersichten aufnehmen.
        Nutzen Sie es später für CRM-Workflows und Reports.
      </p>
    </div>
  </section>

  <section class="panel">
    <div class="panel-head">
      <div>
        <h2>Transaktionen (letzte 50)</h2>
        <p>Schneller Überblick über die zuletzt gespeicherten Verkäufe.</p>
      </div>
      <button disabled title="Export wird später aktiviert">CSV Export</button>
    </div>
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>Beschreibung</th>
            <th>Menge</th>
            <th>Betrag</th>
            <th>Zeitpunkt</th>
          </tr>
        </thead>
        <tbody>
          {#if transactions.length === 0}
            <tr>
              <td colspan="5">Noch keine Transaktionen erfasst.</td>
            </tr>
          {:else}
            {#each transactions as tx}
              <tr>
                <td>{tx.id}</td>
                <td>{tx.description ?? "-"}</td>
                <td>{tx.quantity}</td>
                <td>{formatPrice(tx.total_cents)}</td>
                <td>{new Date(tx.created_at).toLocaleString("de-DE")}</td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </section>
</main>

<style>
:global(body) {
  font-family: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
}

.management-shell {
  min-height: 100vh;
  padding: 1.5rem;
  background: #f5f5f5;
  color: #1d1d1d;
}

header h1 {
  margin: 0 0 0.25rem;
}

header p {
  margin: 0 0 1rem;
  color: #5e5e5e;
}

.panel {
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
  margin-bottom: 1.5rem;
  padding: 1rem 1.25rem;
}

.panel-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.panel-head h2 {
  margin: 0;
}

.panel-head p {
  margin: 0;
  color: #6f6f6f;
  font-size: 0.9rem;
}

.panel-head button {
  border: none;
  border-radius: 4px;
  padding: 0.5rem 0.85rem;
  background: #d5d5d5;
  color: #555;
  cursor: not-allowed;
}

.table-wrapper {
  overflow: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

th,
td {
  padding: 0.6rem 0.5rem;
  border-bottom: 1px solid #ececec;
  text-align: left;
}

th {
  background: #fafafa;
  font-weight: 600;
}

.swatch {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 3px;
  margin-right: 0.4rem;
  vertical-align: middle;
  background: var(--accent, #ccc);
  border: 1px solid rgba(0, 0, 0, 0.08);
}

.placeholder {
  padding: 1rem;
  border: 1px dashed #c8c8c8;
  border-radius: 6px;
  color: #6f6f6f;
  background: #fbfbfb;
}

.status {
  margin-bottom: 1rem;
  color: #5e5e5e;
}

.status.error {
  color: #c62828;
}
</style>
