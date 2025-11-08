<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type Product = {
    id?: number;
    name: string;
    price_cents: number;
    accent?: string | null;
    icon?: string | null;
    note?: string | null;
    product_type_id?: number | null;
    product_type_name?: string | null;
  };

  type Bucket = {
    id: number;
    name: string;
    status: string;
    created_at: string;
    updated_at: string;
    item_count: number;
    total_cents: number;
  };

  type BucketItem = {
    id: number;
    bucket_id: number;
    product_id: number;
    product_name: string;
    quantity: number;
    price_cents: number;
    line_total_cents: number;
    accent?: string | null;
    icon?: string | null;
    note?: string | null;
  };

  type ProductType = {
    id: number;
    name: string;
    color?: string | null;
  };

  type Member = {
    id: number;
    first_name: string;
    last_name: string;
    email?: string | null;
    phone?: string | null;
    status: string;
    notes?: string | null;
  };

  type Membership = {
    id: number;
    member_id: number;
    member_name: string;
    membership_type: string;
    status: string;
    start_date?: string | null;
    end_date?: string | null;
    price_cents?: number | null;
    notes?: string | null;
  };

  type Transaction = {
    id: number;
    product_id?: number | null;
    quantity: number;
    total_cents: number;
    description?: string | null;
    created_at: string;
  };

  type AppSettings = {
    db_location: string;
    language: string;
    currency: string;
    auto_updates: boolean;
    enable_backups: boolean;
  };

  type RoleName = "admin" | "manager" | "user";

  type SessionUser = {
    id: number;
    username: string;
    displayName: string;
    role: RoleName;
  };

  type UserAccount = {
    id: number;
    username: string;
    displayName: string;
    role: string;
    active: boolean;
    createdAt: string;
    updatedAt: string;
  };

  type RoleOption = {
    id: number;
    name: string;
  };

  type UserFormState = {
    id: number | null;
    username: string;
    displayName: string;
    password: string;
    roleId: number | null;
    active: boolean;
  };

  const navItems = [
    { id: "verwaltung", label: "Kasse", minRole: "user" as RoleName },
    { id: "mitglieder", label: "Mitglieder", minRole: "user" as RoleName },
    { id: "transaktionen", label: "Transaktionen", minRole: "user" as RoleName }
  ];

  const adminTabs = [
    "Produkte",
    "Produkttypen",
    "Mitglieder",
    "Mitgliedschaften",
    "Transaktionen",
    "Benutzer",
    "Einstellungen"
  ];

  const fallbackProducts: Product[] = [
    { name: "Eistee", price_cents: 380, accent: "#83ff4e", icon: "🥤" },
    { name: "Saftschorle", price_cents: 350, accent: "#84ff4f", icon: "🧃" },
    { name: "Fruchtsaft", price_cents: 390, accent: "#9bff57", icon: "🍹" },
    { name: "Hauslimonade", price_cents: 350, accent: "#9dff5d", icon: "🍋" },
    { name: "Mineralwasser", price_cents: 200, accent: "#acf5ff", icon: "🚰" },
    { name: "Apfelsaft", price_cents: 340, accent: "#6ded47", icon: "🍏" },
    { name: "Flat White", price_cents: 360, accent: "#33a55d", icon: "☕" },
    { name: "Cappuccino", price_cents: 320, accent: "#2f9755", icon: "🥛" },
    { name: "Latte Macchiato", price_cents: 380, accent: "#2d8b50", icon: "🧋" },
    { name: "Affogato", price_cents: 420, accent: "#297d47", icon: "🍨" },
    { name: "Brezel", price_cents: 250, accent: "#f73ccb", icon: "🥨" },
    { name: "Käsekuchen", price_cents: 340, accent: "#ff46d3", icon: "🍰" },
    { name: "Brownie", price_cents: 290, accent: "#ff40c8", icon: "🧁" },
    { name: "Schokoriegel", price_cents: 150, accent: "#d81b60", icon: "🍫" },
    { name: "Nachos", price_cents: 410, accent: "#e53935", icon: "🌮" },
    { name: "Pizzaschnecke", price_cents: 380, accent: "#c62828", icon: "🍕" },
    { name: "Focaccia", price_cents: 390, accent: "#b71c1c", icon: "🥙" },
    { name: "Tagesangebot", price_cents: 550, accent: "#f44336", icon: "⭐", note: "Chefwahl" }
  ];

  let products: Product[] = fallbackProducts;
  let productTypes: ProductType[] = [];
  let members: Member[] = [];
  let memberships: Membership[] = [];
  let transactions: Transaction[] = [];
  let transactionsToday: Transaction[] = [];

  let usingFallback = true;
  let activeNav: "verwaltung" | "mitglieder" | "transaktionen" = "verwaltung";

  let currentUser: SessionUser | null = null;
  let loginForm = { username: "", password: "" };
  let loginError = "";

  let buckets: Bucket[] = [];
  let activeBucketId: number | null = null;
  let bucketItems: BucketItem[] = [];
  let bucketsLoading = true;
  let bucketItemsLoading = false;

  let showAdminModal = false;
  let activeAdminTab = 0;
  let navMessage = "";

  const blankProductForm = () => ({
    id: null as number | null,
    name: "",
    price_cents: 300,
    accent: "",
    icon: "",
    note: "",
    product_type_id: null as number | null
  });
  let productForm = blankProductForm();

  const blankProductTypeForm = () => ({
    id: null as number | null,
    name: "",
    color: ""
  });
  let productTypeForm = blankProductTypeForm();

  const blankMemberForm = () => ({
    id: null as number | null,
    first_name: "",
    last_name: "",
    email: "",
    phone: "",
    status: "active",
    notes: ""
  });
  let memberForm = blankMemberForm();

  const blankMembershipForm = () => ({
    id: null as number | null,
    member_id: members[0]?.id ?? 0,
    membership_type: "",
    status: "active",
    start_date: "",
    end_date: "",
    price_cents: 0,
    notes: ""
  });
  let membershipForm = blankMembershipForm();
  const blankSettingsForm = (): AppSettings => ({
    db_location: "",
    language: "de",
    currency: "EUR",
    auto_updates: true,
    enable_backups: false
  });
  let settingsForm: AppSettings = blankSettingsForm();
  let settingsLoading = false;
  let settingsMessage = "";
  let users: UserAccount[] = [];
  let roles: RoleOption[] = [];
  const blankUserForm = (): UserFormState => ({
    id: null,
    username: "",
    displayName: "",
    password: "",
    roleId: roles[0]?.id ?? null,
    active: true
  });
  let userForm: UserFormState = blankUserForm();
  let userMessage = "";
  const rolePriority: Record<RoleName, number> = { admin: 3, manager: 2, user: 1 };
  const mockBuckets: Bucket[] = [
    {
      id: 9001,
      name: "Demo Tisch 1",
      status: "open",
      created_at: "2024-01-01T10:00:00Z",
      updated_at: "2024-01-01T10:05:00Z",
      item_count: 2,
      total_cents: 860
    },
    {
      id: 9002,
      name: "Demo Theke",
      status: "open",
      created_at: "2024-01-01T09:30:00Z",
      updated_at: "2024-01-01T09:45:00Z",
      item_count: 1,
      total_cents: 420
    }
  ];

  const mockBucketItems: BucketItem[] = [
    {
      id: 9101,
      bucket_id: 9001,
      product_id: 1,
      product_name: "Latte Demo",
      quantity: 2,
      price_cents: 320,
      line_total_cents: 640,
      accent: "#33a55d",
      icon: "☕",
      note: null
    },
    {
      id: 9102,
      bucket_id: 9001,
      product_id: 2,
      product_name: "Cookie Demo",
      quantity: 1,
      price_cents: 220,
      line_total_cents: 220,
      accent: "#ff40c8",
      icon: "🍪",
      note: null
    }
  ];

  const mockMembers: Member[] = [
    {
      id: 8001,
      first_name: "Demo",
      last_name: "Mitglied",
      email: "demo@example.com",
      phone: "+49 000 000",
      status: "active",
      notes: "Testzugang"
    }
  ];

  const mockTransactionsToday: Transaction[] = [
    {
      id: 7001,
      product_id: null,
      quantity: 1,
      total_cents: 860,
      description: "Demo Bestellung",
      created_at: "2024-01-01T11:00:00Z"
    }
  ];

  $: visibleAdminTabs = adminTabs.filter((tab) => {
    if (!currentUser) return false;
    if (currentUser.role === "user") return false;
    if (currentUser.role === "manager" && (tab === "Benutzer" || tab === "Einstellungen")) {
      return false;
    }
    return true;
  });
  $: currentAdminTab = visibleAdminTabs[activeAdminTab] ?? null;
  $: if (activeAdminTab >= visibleAdminTabs.length) {
    activeAdminTab = 0;
  }

  $: displayProducts = currentUser ? products : fallbackProducts;
  $: displayBuckets = currentUser ? buckets : mockBuckets;
  $: displayBucketsLoading = currentUser ? bucketsLoading : false;
  $: displayBucketItems = currentUser ? bucketItems : mockBucketItems;
  $: displayMembers = currentUser ? members : mockMembers;
  $: displayTransactionsToday = currentUser ? transactionsToday : mockTransactionsToday;
  $: displayBucketItemsLoading = currentUser ? bucketItemsLoading : false;
  $: displayActiveBucketId =
    currentUser ? activeBucketId : displayBuckets[0]?.id ?? null;
  $: activeBucketDisplay =
    displayBuckets.find((bucket) => bucket.id === displayActiveBucketId) ?? null;
  $: bucketTotal = displayBucketItems.reduce(
    (sum, item) => sum + (item.line_total_cents ?? item.price_cents * item.quantity),
    0
  );

  onMount(() => {
    loadProducts();
    initializeBuckets();
  });

  async function loadProducts() {
    try {
      const stored = await invoke<Product[]>("list_products");
      if (Array.isArray(stored) && stored.length > 0) {
        products = stored;
        usingFallback = false;
      }
    } catch (error) {
      console.error("Konnte Produkte nicht laden", error);
    }
  }

  async function initializeBuckets() {
    await loadBuckets();
    if (activeBucketId) {
      await loadBucketItems(activeBucketId);
    }
  }

  async function loadBuckets(selectBucketId?: number) {
    bucketsLoading = true;
    try {
      const data = await invoke<Bucket[]>("list_buckets");
      buckets = data ?? [];
      if (selectBucketId) {
        activeBucketId = selectBucketId;
      } else if (activeBucketId && !buckets.some((bucket) => bucket.id === activeBucketId)) {
        activeBucketId = buckets[0]?.id ?? null;
      } else if (!activeBucketId && buckets.length > 0) {
        activeBucketId = buckets[0].id;
      }
    } catch (error) {
      console.error("Buckets konnten nicht geladen werden", error);
    } finally {
      bucketsLoading = false;
    }
  }

  async function loadBucketItems(bucketId: number | null) {
    if (!bucketId) {
      bucketItems = [];
      return;
    }
    bucketItemsLoading = true;
    try {
      const items = await invoke<BucketItem[]>("get_bucket_items", { payload: { bucketId } });
      bucketItems =
        items?.map((item: any) => ({
          ...item,
          product_name: item.product_name ?? item.productName ?? "",
          price_cents: item.price_cents ?? item.priceCents ?? 0,
          line_total_cents: item.line_total_cents ?? item.lineTotalCents ?? 0
        })) ?? [];
    } catch (error) {
      console.error("Bucket-Inhalte konnten nicht geladen werden", error);
      bucketItems = [];
    } finally {
      bucketItemsLoading = false;
    }
  }

  async function ensureBucketSelected(): Promise<number | null> {
    if (activeBucketId) {
      return activeBucketId;
    }
    const id = await createBucketWithName();
    return id;
  }

  async function createBucketWithName(name?: string | null) {
    try {
      const payload = name && name.trim().length > 0 ? { name: name.trim() } : undefined;
      const id = await invoke<number>("create_bucket", payload ? { payload } : {});
      await loadBuckets(id);
      if (id) {
        await loadBucketItems(id);
      }
      return id;
    } catch (error) {
      console.error("Bucket konnte nicht erstellt werden", error);
      return null;
    }
  }

  async function handleCreateBucket() {
    const suggestion = `Bucket ${buckets.length + 1}`;
    const name = window.prompt("Neuen Bucket anlegen", suggestion);
    await createBucketWithName(name ?? undefined);
  }

  async function handleRenameBucket(bucketId: number) {
    const bucket = buckets.find((b) => b.id === bucketId);
    const name = window.prompt("Bucket umbenennen", bucket?.name ?? "");
    if (!name || !name.trim() || name.trim() === bucket?.name) {
      return;
    }
    try {
      await invoke("rename_bucket", { payload: { id: bucketId, name: name.trim() } });
      await loadBuckets(bucketId);
      await loadBucketItems(bucketId);
    } catch (error) {
      console.error("Bucket konnte nicht umbenannt werden", error);
    }
  }

  async function handleBucketSelect(bucketId: number) {
    activeBucketId = bucketId;
    await loadBucketItems(bucketId);
  }

  function formatPrice(cents: number) {
    return new Intl.NumberFormat("de-DE", {
      style: "currency",
      currency: "EUR"
    }).format((cents ?? 0) / 100);
  }

  function euroInputToCents(value: string) {
    const parsed = Number.parseFloat(value ?? "0");
    if (Number.isNaN(parsed)) {
      return 0;
    }
    return Math.round(parsed * 100);
  }

  async function handleProductClick(product: Product) {
    if (!product.id) {
      console.warn("Produkt hat keine ID und kann nicht gebucht werden.");
      return;
    }

    const bucketId = await ensureBucketSelected();
    if (!bucketId) {
      return;
    }

    try {
      await invoke("add_product_to_bucket", {
        payload: {
          bucketId,
          productId: product.id,
          quantity: 1
        }
      });
      await loadBucketItems(bucketId);
      await loadBuckets(bucketId);
    } catch (error) {
      console.error("Produkt konnte nicht zum Bucket hinzugefügt werden", error);
    }
  }

  function openAdminModal() {
    if (!currentUser || currentUser.role === "user") {
      navMessage = "Keine Berechtigung für Kasse.";
      return;
    }
    showAdminModal = true;
    activeAdminTab = 0;
    refreshAdminTab(0);
  }

  function closeAdminModal() {
    showAdminModal = false;
  }

  async function refreshAdminTab(tabIndex: number) {
    const tab = visibleAdminTabs[tabIndex];
    if (!tab) return;
    switch (tab) {
      case "Produkte":
        await Promise.all([loadProducts(), loadProductTypes()]);
        break;
      case "Produkttypen":
        await loadProductTypes();
        break;
      case "Mitglieder":
        await loadMembers();
        break;
      case "Mitgliedschaften":
        await Promise.all([loadMembers(), loadMemberships()]);
        break;
      case "Transaktionen":
        await loadTransactions();
        break;
      case "Benutzer":
        await Promise.all([loadRoles(), loadUsers()]);
        break;
      case "Einstellungen":
        await loadSettings();
        break;
    }
  }

  async function handleAdminTabChange(index: number) {
    activeAdminTab = index;
    await refreshAdminTab(index);
  }

  async function loadProductTypes() {
    try {
      const result = await invoke<ProductType[]>("list_product_types");
      productTypes = result ?? [];
      if (productTypes.length > 0 && !productForm.product_type_id) {
        productForm.product_type_id = productTypes[0].id;
      }
    } catch (error) {
      console.error("Produkttypen konnten nicht geladen werden", error);
    }
  }

  async function submitProduct() {
    if (!productForm.name.trim()) {
      return;
    }
    try {
      await invoke("save_product", { payload: productForm });
      productForm = blankProductForm();
      await loadProducts();
    } catch (error) {
      console.error("Produkt konnte nicht gespeichert werden", error);
    }
  }

  function editProduct(product: Product) {
    productForm = {
      id: product.id ?? null,
      name: product.name,
      price_cents: product.price_cents,
      accent: product.accent ?? "",
      icon: product.icon ?? "",
      note: product.note ?? "",
      product_type_id: product.product_type_id ?? null
    };
  }

  async function removeProduct(id: number | undefined) {
    if (!id) return;
    if (!window.confirm("Produkt wirklich löschen?")) return;
    try {
      await invoke("delete_product", { id });
      if (productForm.id === id) {
        productForm = blankProductForm();
      }
      await loadProducts();
    } catch (error) {
      console.error("Produkt konnte nicht gelöscht werden", error);
    }
  }

  async function submitProductType() {
    if (!productTypeForm.name.trim()) return;
    try {
      await invoke("save_product_type", { payload: productTypeForm });
      productTypeForm = blankProductTypeForm();
      await loadProductTypes();
    } catch (error) {
      console.error("Produkttyp konnte nicht gespeichert werden", error);
    }
  }

  function editProductType(type: ProductType) {
    productTypeForm = {
      id: type.id,
      name: type.name,
      color: type.color ?? ""
    };
  }

  async function removeProductType(id: number) {
    if (!window.confirm("Produkttyp löschen? Zugeordnete Produkte behalten die Referenz.")) return;
    try {
      await invoke("delete_product_type", { id });
      if (productTypeForm.id === id) {
        productTypeForm = blankProductTypeForm();
      }
      await loadProductTypes();
      await loadProducts();
    } catch (error) {
      console.error("Produkttyp konnte nicht gelöscht werden", error);
    }
  }

  async function loadMembers() {
    try {
      const data = await invoke<Member[]>("list_members");
      members = data ?? [];
      if (!membershipForm.member_id && members.length > 0) {
        membershipForm.member_id = members[0].id;
      }
    } catch (error) {
      console.error("Mitglieder konnten nicht geladen werden", error);
    }
  }

  async function submitMember() {
    if (!memberForm.first_name.trim() || !memberForm.last_name.trim()) return;
    try {
      await invoke("save_member", { payload: memberForm });
      memberForm = blankMemberForm();
      await loadMembers();
    } catch (error) {
      console.error("Mitglied konnte nicht gespeichert werden", error);
    }
  }

  function editMember(member: Member) {
    memberForm = {
      id: member.id,
      first_name: member.first_name,
      last_name: member.last_name,
      email: member.email ?? "",
      phone: member.phone ?? "",
      status: member.status,
      notes: member.notes ?? ""
    };
  }

  async function removeMember(id: number) {
    if (!window.confirm("Mitglied löschen? Zugehörige Mitgliedschaften werden ebenfalls gelöscht.")) return;
    try {
      await invoke("delete_member", { id });
      if (memberForm.id === id) {
        memberForm = blankMemberForm();
      }
      await Promise.all([loadMembers(), loadMemberships()]);
    } catch (error) {
      console.error("Mitglied konnte nicht gelöscht werden", error);
    }
  }

  async function loadMemberships() {
    try {
      const data = await invoke<Membership[]>("list_memberships");
      memberships = data ?? [];
    } catch (error) {
      console.error("Mitgliedschaften konnten nicht geladen werden", error);
    }
  }

  async function submitMembership() {
    if (!membershipForm.member_id || !membershipForm.membership_type.trim()) return;
    try {
      await invoke("save_membership", { payload: membershipForm });
      membershipForm = blankMembershipForm();
      await loadMemberships();
    } catch (error) {
      console.error("Mitgliedschaft konnte nicht gespeichert werden", error);
    }
  }

  function editMembership(ms: Membership) {
    membershipForm = {
      id: ms.id,
      member_id: ms.member_id,
      membership_type: ms.membership_type,
      status: ms.status,
      start_date: ms.start_date ?? "",
      end_date: ms.end_date ?? "",
      price_cents: ms.price_cents ?? 0,
      notes: ms.notes ?? ""
    };
  }

  async function removeMembership(id: number) {
    if (!window.confirm("Mitgliedschaft löschen?")) return;
    try {
      await invoke("delete_membership", { id });
      if (membershipForm.id === id) {
        membershipForm = blankMembershipForm();
      }
      await loadMemberships();
    } catch (error) {
      console.error("Mitgliedschaft konnte nicht gelöscht werden", error);
    }
  }

  async function loadTransactions() {
    try {
      const data = await invoke<Transaction[]>("list_transactions");
      transactions = data ?? [];
    } catch (error) {
      console.error("Transaktionen konnten nicht geladen werden", error);
    }
  }

  async function loadTransactionsToday() {
    try {
      const data = await invoke<Transaction[]>("list_transactions_today");
      transactionsToday = data ?? [];
    } catch (error) {
      console.error("Heutige Transaktionen konnten nicht geladen werden", error);
    }
  }

  async function removeTransaction(id: number) {
    if (!window.confirm("Transaktion löschen?")) return;
    try {
      await invoke("delete_transaction", { id });
      await loadTransactions();
    } catch (error) {
      console.error("Transaktion konnte nicht gelöscht werden", error);
    }
  }

  async function loadRoles() {
    try {
      const data = await invoke<RoleOption[]>("list_roles");
      roles = data ?? [];
      if (!userForm.roleId && roles.length > 0) {
        userForm.roleId = roles[0].id;
      }
    } catch (error) {
      console.error("Rollen konnten nicht geladen werden", error);
    }
  }

  async function loadUsers() {
    try {
      const data = await invoke<UserAccount[]>("list_users");
      users = data ?? [];
    } catch (error) {
      console.error("Benutzer konnten nicht geladen werden", error);
    }
  }

  async function submitUser(event?: Event) {
    event?.preventDefault();
    userMessage = "";
    if (!userForm.username.trim() || !userForm.displayName.trim() || !userForm.roleId) {
      userMessage = "Bitte alle Pflichtfelder ausfüllen.";
      return;
    }
    if (!userForm.id && !userForm.password.trim()) {
      userMessage = "Passwort erforderlich.";
      return;
    }
    try {
      const payload = {
        id: userForm.id,
        username: userForm.username,
        displayName: userForm.displayName,
        password: userForm.password.trim() ? userForm.password : null,
        roleId: userForm.roleId,
        active: userForm.active
      };
      await invoke("save_user", { payload });
      userMessage = "Gespeichert.";
      userForm = blankUserForm();
      await loadUsers();
    } catch (error) {
      console.error("Benutzer konnte nicht gespeichert werden", error);
      userMessage = "Fehler beim Speichern.";
    }
  }

  function editUser(user: UserAccount) {
    const role = roles.find((r) => r.name === user.role);
    userForm = {
      id: user.id,
      username: user.username,
      displayName: user.displayName,
      password: "",
      roleId: role?.id ?? null,
      active: user.active
    };
  }

  async function removeUser(id: number) {
    if (!window.confirm("Benutzer löschen?")) return;
    try {
      await invoke("delete_user", { id });
      if (userForm.id === id) {
        userForm = blankUserForm();
      }
      await loadUsers();
    } catch (error) {
      console.error("Benutzer konnte nicht gelöscht werden", error);
    }
  }

  async function loadSettings() {
    settingsLoading = true;
    settingsMessage = "";
    try {
      const loaded = await invoke<AppSettings>("get_settings");
      settingsForm = { ...loaded };
    } catch (error) {
      console.error("Einstellungen konnten nicht geladen werden", error);
      settingsMessage = "Fehler beim Laden der Einstellungen.";
    } finally {
      settingsLoading = false;
    }
  }

  async function submitSettings(event?: Event) {
    event?.preventDefault();
    settingsMessage = "";
    try {
      const saved = await invoke<AppSettings>("update_settings", { payload: settingsForm });
      settingsForm = { ...saved };
      settingsMessage = "Einstellungen gespeichert.";
    } catch (error) {
      console.error("Einstellungen konnten nicht gespeichert werden", error);
      settingsMessage = "Speichern fehlgeschlagen.";
    }
  }

  function roleMeets(required: RoleName, actual: RoleName) {
    return rolePriority[actual] >= rolePriority[required];
  }

  function canAccessNav(itemId: "verwaltung" | "mitglieder" | "transaktionen") {
    if (!currentUser) return false;
    const def = navItems.find((n) => n.id === itemId);
    if (!def) return false;
    return roleMeets(def.minRole, currentUser.role);
  }

  async function handleNavChange(itemId: "verwaltung" | "mitglieder" | "transaktionen") {
    navMessage = "";
    if (!canAccessNav(itemId)) {
      navMessage = "Keine Berechtigung für diesen Bereich.";
      return;
    }
    activeNav = itemId;
    if (itemId === "mitglieder") {
      await loadMembers();
    } else if (itemId === "transaktionen") {
      await loadTransactionsToday();
    }
  }

  async function handleLogin(event?: Event) {
    event?.preventDefault();
    loginError = "";
    try {
      const user = await invoke<SessionUser>("login_user", {
        payload: { username: loginForm.username, password: loginForm.password }
      });
      currentUser = user;
      loginForm = { username: "", password: "" };
      if (!canAccessNav(activeNav)) {
        activeNav = currentUser?.role === "user" ? "mitglieder" : "verwaltung";
      }
    } catch (error) {
      console.error("Login fehlgeschlagen", error);
      loginError = "Anmeldung nicht möglich.";
    }
  }

  function logout() {
    currentUser = null;
    showAdminModal = false;
    activeNav = "verwaltung";
  }

  $: if (currentUser && !canAccessNav(activeNav)) {
    activeNav = currentUser.role === "user" ? "mitglieder" : "verwaltung";
  }
</script>

{#if !currentUser}
  <div class="login-overlay">
    <form class="login-card" on:submit|preventDefault={handleLogin}>
      <h2>Anmeldung</h2>
      <label>
        Benutzername
        <input bind:value={loginForm.username} autocomplete="username" required />
      </label>
      <label>
        Passwort
        <input type="password" bind:value={loginForm.password} autocomplete="current-password" required />
      </label>
      {#if loginError}
        <p class="settings-message error">{loginError}</p>
      {/if}
      <button type="submit">Einloggen</button>
    </form>
  </div>
{/if}

<main class={`pos-app ${!currentUser ? "blurred-bg" : ""}`}>
  <header class="top-bar">
    <div class="app-title">Boulderado POS</div>
    <nav class="tab-strip">
      {#each navItems as item}
        <button
          class:active={activeNav === item.id}
          disabled={!currentUser || !canAccessNav(item.id)}
          on:click={() => handleNavChange(item.id)}
        >
          {item.label}
        </button>
      {/each}
    </nav>
    {#if currentUser && currentUser.role !== "user"}
      <button class="admin-btn" type="button" on:click={openAdminModal}>Verwaltungsmenü</button>
    {/if}
    <div class="status-box">
      {#if currentUser}
        <span class="label">Angemeldet als</span>
        <strong>{currentUser.displayName} ({currentUser.role})</strong>
        <button class="linkish" type="button" on:click={logout}>Logout</button>
      {:else}
        <span class="label">Bitte anmelden</span>
      {/if}
    </div>
  </header>

  {#if navMessage}
    <p class="nav-message">{navMessage}</p>
  {/if}

  {#if activeNav === "verwaltung"}
    <section class="content-area">
      <aside class="bucket-panel">
      <div class="panel-title">Buckets</div>
      <div class="bucket-actions">
        <button type="button" class="create-bucket" on:click={handleCreateBucket}>
          + Neuer Bucket
        </button>
      </div>
      <div class="bucket-list">
        {#if displayBucketsLoading}
          <p class="muted">Lade Buckets...</p>
        {:else if displayBuckets.length === 0}
          <p class="muted">Noch keine Buckets vorhanden.</p>
        {:else}
          {#each displayBuckets as bucket}
            <button
              type="button"
              class="bucket-row"
              class:active={bucket.id === displayActiveBucketId}
              on:click={() => currentUser && handleBucketSelect(bucket.id)}
            >
              <div class="bucket-row-main">
                <span class="bucket-name">{bucket.name}</span>
                <span class="bucket-meta">
                  {bucket.item_count} · {formatPrice(bucket.total_cents)}
                </span>
              </div>
              <span
                class="rename-bucket"
                title="Umbenennen"
                on:click|stopPropagation={() => handleRenameBucket(bucket.id)}
              >
                ✎
              </span>
            </button>
          {/each}
        {/if}
      </div>
    </aside>

    <div class="grid-panel">
      <div class="info-bar">
        <span>Letzte Buchung: <strong>14:32 Uhr</strong></span>
        <span>Bediener: <strong>Alex</strong></span>
        <span>Aktiver Bucket: <strong>{activeBucketDisplay?.name ?? "—"}</strong></span>
        <span>Datenquelle: <strong>{usingFallback ? "Demo" : "SQLite"}</strong></span>
      </div>

      <div class="product-grid">
        {#each displayProducts as product}
          <button
            class="product-card"
            style={`--accent: ${product.accent ?? "#fefefe"};`}
            type="button"
            on:click={() => handleProductClick(product)}
          >
            <span class="icon" aria-hidden="true">{product.icon ?? "🍽️"}</span>
            <span class="name">{product.name}</span>
            {#if product.note}
              <span class="note">{product.note}</span>
            {/if}
            <span class="price">{formatPrice(product.price_cents)}</span>
          </button>
        {/each}
      </div>
    </div>

    <aside class="receipt-panel">
      <div class="panel-title">Bon</div>
      <div class="receipt-content">
        {#if !displayActiveBucketId}
          <p class="muted">Kein Bucket ausgewählt.</p>
        {:else if displayBucketItemsLoading}
          <p class="muted">Lade Positionen...</p>
        {:else if displayBucketItems.length === 0}
          <p class="muted">Bucket ist leer.</p>
        {:else}
          <div class="ticket-lines">
            {#each displayBucketItems as item}
              <div class="ticket-line">
                <div>
                  <span class="qty">{item.quantity}×</span>
                  <span>{item.product_name}</span>
                </div>
                <strong>{formatPrice(item.line_total_cents)}</strong>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      <div class="receipt-footer">
        <div>
          <strong>{activeBucketDisplay?.name ?? "Kein Bucket"}</strong>
          <small>{displayBucketItems.length} Positionen</small>
        </div>
        <div class="total">{formatPrice(bucketTotal)}</div>
      </div>
      <button
        class="checkout"
        type="button"
        disabled={
          !currentUser ||
          !activeBucketId ||
          bucketItems.length === 0 ||
          bucketItemsLoading
        }
      >
        Zahlung starten
      </button>
    </aside>
    </section>
  {:else if activeNav === "mitglieder"}
    <section class="nav-panel">
      <div class="admin-grid">
        <form class="admin-form" on:submit|preventDefault={submitMember}>
          <h3>{memberForm.id ? "Mitglied bearbeiten" : "Mitglied anlegen"}</h3>
          <label>
            Vorname
            <input bind:value={memberForm.first_name} required />
          </label>
          <label>
            Nachname
            <input bind:value={memberForm.last_name} required />
          </label>
          <label>
            E-Mail
            <input type="email" bind:value={memberForm.email} />
          </label>
          <label>
            Telefon
            <input bind:value={memberForm.phone} />
          </label>
          <label>
            Status
            <select bind:value={memberForm.status}>
              <option value="active">Aktiv</option>
              <option value="inactive">Inaktiv</option>
              <option value="blocked">Gesperrt</option>
            </select>
          </label>
          <label>
            Notizen
            <textarea rows="2" bind:value={memberForm.notes}></textarea>
          </label>
          <div class="form-actions">
            <button type="submit">{memberForm.id ? "Aktualisieren" : "Speichern"}</button>
            <button type="button" on:click={() => (memberForm = blankMemberForm())}>Zurücksetzen</button>
          </div>
        </form>
        <div class="admin-table">
          <h3>Mitglieder</h3>
          <table>
            <thead>
              <tr>
                <th>Name</th>
                <th>E-Mail</th>
                <th>Status</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each displayMembers as member}
                <tr>
                  <td>{member.first_name} {member.last_name}</td>
                  <td>{member.email ?? "—"}</td>
                  <td>{member.status}</td>
                  <td class="actions">
                    <button type="button" on:click={() => editMember(member)}>Bearbeiten</button>
                    <button type="button" on:click={() => removeMember(member.id)}>Löschen</button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  {:else}
    <section class="nav-panel">
      <div class="admin-table">
        <h3>Heutige Transaktionen</h3>
        <table>
          <thead>
            <tr>
              <th>#</th>
              <th>Beschreibung</th>
              <th>Menge</th>
              <th>Betrag</th>
              <th>Zeit</th>
            </tr>
          </thead>
          <tbody>
            {#if displayTransactionsToday.length === 0}
              <tr>
                <td colspan="5">Keine Transaktionen für heute.</td>
              </tr>
            {:else}
              {#each displayTransactionsToday as tx}
                <tr>
                  <td>{tx.id}</td>
                  <td>{tx.description ?? "—"}</td>
                  <td>{tx.quantity}</td>
                  <td>{formatPrice(tx.total_cents)}</td>
                  <td>{new Date(tx.created_at).toLocaleTimeString("de-DE")}</td>
                </tr>
              {/each}
            {/if}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</main>

{#if showAdminModal}
  <div class="modal-overlay" on:click={closeAdminModal}>
    <div class="modal" on:click|stopPropagation>
      <header class="modal-header">
        <h2>Verwaltung</h2>
        <button class="close-btn" type="button" on:click={closeAdminModal}>×</button>
      </header>
      <nav class="modal-tabs">
        {#each visibleAdminTabs as tab, index}
          <button class:active={index === activeAdminTab} on:click={() => handleAdminTabChange(index)}>
            {tab}
          </button>
        {/each}
      </nav>

      <section class="modal-body">
        {#if currentAdminTab === "Produkte"}
          <div class="admin-grid">
            <form class="admin-form" on:submit|preventDefault={submitProduct}>
              <h3>{productForm.id ? "Produkt bearbeiten" : "Produkt anlegen"}</h3>
              <label>
                Name
                <input bind:value={productForm.name} required />
              </label>
              <label>
                Preis (€)
                <input
                  type="number"
                  step="0.01"
                  value={productForm.price_cents / 100}
                  on:input={(event) => (productForm.price_cents = euroInputToCents((event.currentTarget as HTMLInputElement).value))}
                  required
                />
              </label>
              <label>
                Produkttyp
                <select
                  value={productForm.product_type_id ?? ""}
                  on:change={(event) => {
                    const value = (event.currentTarget as HTMLSelectElement).value;
                    productForm.product_type_id = value ? Number(value) : null;
                  }}
                >
                  <option value="">-</option>
                  {#each productTypes as type}
                    <option value={type.id}>{type.name}</option>
                  {/each}
                </select>
              </label>
              <label>
                Akzentfarbe
                <input type="color" bind:value={productForm.accent} />
              </label>
              <label>
                Icon (Emoji)
                <input bind:value={productForm.icon} placeholder="🥤" />
              </label>
              <label>
                Notiz
                <input bind:value={productForm.note} />
              </label>
              <div class="form-actions">
                <button type="submit">{productForm.id ? "Aktualisieren" : "Speichern"}</button>
                <button type="button" on:click={() => (productForm = blankProductForm())}>Zurücksetzen</button>
              </div>
            </form>

            <div class="admin-table">
              <h3>Produkte</h3>
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Typ</th>
                    <th>Preis</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each products as product}
                    <tr>
                      <td>{product.name}</td>
                      <td>{product.product_type_name ?? "—"}</td>
                      <td>{formatPrice(product.price_cents)}</td>
                      <td class="actions">
                        <button type="button" on:click={() => editProduct(product)}>Bearbeiten</button>
                        <button type="button" on:click={() => removeProduct(product.id)}>Löschen</button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else if currentAdminTab === "Produkttypen"}
          <div class="admin-grid">
            <form class="admin-form" on:submit|preventDefault={submitProductType}>
              <h3>{productTypeForm.id ? "Produkttyp bearbeiten" : "Produkttyp anlegen"}</h3>
              <label>
                Name
                <input bind:value={productTypeForm.name} required />
              </label>
              <label>
                Farbe
                <input type="color" bind:value={productTypeForm.color} />
              </label>
              <div class="form-actions">
                <button type="submit">{productTypeForm.id ? "Aktualisieren" : "Speichern"}</button>
                <button type="button" on:click={() => (productTypeForm = blankProductTypeForm())}>Zurücksetzen</button>
              </div>
            </form>
            <div class="admin-table">
              <h3>Produkttypen</h3>
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Farbe</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each productTypes as type}
                    <tr>
                      <td>{type.name}</td>
                      <td>
                        {#if type.color}
                          <span class="swatch" style={`--accent:${type.color}`}></span>
                          {type.color}
                        {:else}
                          —
                        {/if}
                      </td>
                      <td class="actions">
                        <button type="button" on:click={() => editProductType(type)}>Bearbeiten</button>
                        <button type="button" on:click={() => removeProductType(type.id)}>Löschen</button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else if currentAdminTab === "Mitglieder"}
          <div class="admin-grid">
            <form class="admin-form" on:submit|preventDefault={submitMember}>
              <h3>{memberForm.id ? "Mitglied bearbeiten" : "Mitglied anlegen"}</h3>
              <label>
                Vorname
                <input bind:value={memberForm.first_name} required />
              </label>
              <label>
                Nachname
                <input bind:value={memberForm.last_name} required />
              </label>
              <label>
                E-Mail
                <input type="email" bind:value={memberForm.email} />
              </label>
              <label>
                Telefon
                <input bind:value={memberForm.phone} />
              </label>
              <label>
                Status
                <select bind:value={memberForm.status}>
                  <option value="active">Aktiv</option>
                  <option value="inactive">Inaktiv</option>
                  <option value="blocked">Gesperrt</option>
                </select>
              </label>
              <label>
                Notizen
                <textarea rows="2" bind:value={memberForm.notes}></textarea>
              </label>
              <div class="form-actions">
                <button type="submit">{memberForm.id ? "Aktualisieren" : "Speichern"}</button>
                <button type="button" on:click={() => (memberForm = blankMemberForm())}>Zurücksetzen</button>
              </div>
            </form>
            <div class="admin-table">
              <h3>Mitglieder</h3>
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>E-Mail</th>
                    <th>Status</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each members as member}
                    <tr>
                      <td>{member.first_name} {member.last_name}</td>
                      <td>{member.email ?? "—"}</td>
                      <td>{member.status}</td>
                      <td class="actions">
                        <button type="button" on:click={() => editMember(member)}>Bearbeiten</button>
                        <button type="button" on:click={() => removeMember(member.id)}>Löschen</button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else if currentAdminTab === "Mitgliedschaften"}
          <div class="admin-grid">
            <form class="admin-form" on:submit|preventDefault={submitMembership}>
              <h3>{membershipForm.id ? "Mitgliedschaft bearbeiten" : "Mitgliedschaft anlegen"}</h3>
              <label>
                Mitglied
                <select
                  value={membershipForm.member_id || ""}
                  on:change={(event) => {
                    const value = (event.currentTarget as HTMLSelectElement).value;
                    membershipForm.member_id = value ? Number(value) : 0;
                  }}
                >
                  {#if members.length === 0}
                    <option value="" disabled>Keine Mitglieder vorhanden</option>
                  {:else}
                    {#each members as member}
                      <option value={member.id}>{member.first_name} {member.last_name}</option>
                    {/each}
                  {/if}
                </select>
              </label>
              <label>
                Typ
                <input bind:value={membershipForm.membership_type} required />
              </label>
              <label>
                Status
                <select bind:value={membershipForm.status}>
                  <option value="active">Aktiv</option>
                  <option value="paused">Pausiert</option>
                  <option value="cancelled">Beendet</option>
                </select>
              </label>
              <label>
                Startdatum
                <input type="date" bind:value={membershipForm.start_date} />
              </label>
              <label>
                Enddatum
                <input type="date" bind:value={membershipForm.end_date} />
              </label>
              <label>
                Preis (€)
                <input
                  type="number"
                  step="0.01"
                  value={(membershipForm.price_cents ?? 0) / 100}
                  on:input={(event) =>
                    (membershipForm.price_cents = euroInputToCents(
                      (event.currentTarget as HTMLInputElement).value
                    ))
                  }
                />
              </label>
              <label>
                Notizen
                <textarea rows="2" bind:value={membershipForm.notes}></textarea>
              </label>
              <div class="form-actions">
                <button type="submit">{membershipForm.id ? "Aktualisieren" : "Speichern"}</button>
                <button type="button" on:click={() => (membershipForm = blankMembershipForm())}>Zurücksetzen</button>
              </div>
            </form>
            <div class="admin-table">
              <h3>Mitgliedschaften</h3>
              <table>
                <thead>
                  <tr>
                    <th>Mitglied</th>
                    <th>Typ</th>
                    <th>Status</th>
                    <th>Laufzeit</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each memberships as ms}
                    <tr>
                      <td>{ms.member_name}</td>
                      <td>{ms.membership_type}</td>
                      <td>{ms.status}</td>
                      <td>
                        {ms.start_date ?? "—"} – {ms.end_date ?? "—"}
                      </td>
                      <td class="actions">
                        <button type="button" on:click={() => editMembership(ms)}>Bearbeiten</button>
                        <button type="button" on:click={() => removeMembership(ms.id)}>Löschen</button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else if currentAdminTab === "Transaktionen"}
          <div class="admin-table">
            <h3>Transaktionen</h3>
            <table>
              <thead>
                <tr>
                  <th>#</th>
                  <th>Beschreibung</th>
                  <th>Menge</th>
                  <th>Betrag</th>
                  <th>Zeit</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {#each transactions as tx}
                  <tr>
                    <td>{tx.id}</td>
                    <td>{tx.description ?? "—"}</td>
                    <td>{tx.quantity}</td>
                    <td>{formatPrice(tx.total_cents)}</td>
                    <td>{new Date(tx.created_at).toLocaleString("de-DE")}</td>
                    <td class="actions">
                      <button type="button" on:click={() => removeTransaction(tx.id)}>Löschen</button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else if currentAdminTab === "Benutzer"}
          <div class="admin-grid">
            <form class="admin-form" on:submit|preventDefault={submitUser}>
              <h3>{userForm.id ? "Benutzer bearbeiten" : "Benutzer anlegen"}</h3>
              <label>
                Benutzername
                <input bind:value={userForm.username} required />
              </label>
              <label>
                Anzeigename
                <input bind:value={userForm.displayName} required />
              </label>
              <label>
                Rolle
                <select
                  value={userForm.roleId ?? ""}
                  on:change={(event) => {
                    const value = (event.currentTarget as HTMLSelectElement).value;
                    userForm.roleId = value ? Number(value) : null;
                  }}
                  required
                >
                  <option value="" disabled>Rolle wählen</option>
                  {#each roles as role}
                    <option value={role.id}>{role.name}</option>
                  {/each}
                </select>
              </label>
              <label>
                Passwort {#if userForm.id}<small>(leer lassen, um unverändert zu lassen)</small>{/if}
                <input
                  type="password"
                  bind:value={userForm.password}
                  placeholder={userForm.id ? "Neues Passwort" : "Passwort"}
                  required={!userForm.id}
                />
              </label>
              <label class="switch-row">
                <input type="checkbox" bind:checked={userForm.active} />
                <span>Aktiv</span>
              </label>
              {#if userMessage}
                <p class="muted">{userMessage}</p>
              {/if}
              <div class="form-actions">
                <button type="submit">{userForm.id ? "Aktualisieren" : "Speichern"}</button>
                <button type="button" on:click={() => (userForm = blankUserForm())}>Zurücksetzen</button>
              </div>
            </form>
            <div class="admin-table">
              <h3>Benutzer</h3>
              <table>
                <thead>
                  <tr>
                    <th>Benutzername</th>
                    <th>Name</th>
                    <th>Rolle</th>
                    <th>Status</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each users as user}
                    <tr>
                      <td>{user.username}</td>
                      <td>{user.displayName}</td>
                      <td>{user.role}</td>
                      <td>{user.active ? "aktiv" : "inaktiv"}</td>
                      <td class="actions">
                        <button type="button" on:click={() => editUser(user)}>Bearbeiten</button>
                        {#if user.username !== "admin"}
                          <button type="button" on:click={() => removeUser(user.id)}>Löschen</button>
                        {/if}
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else}
          <form class="settings-form" on:submit|preventDefault={submitSettings}>
            <div class="settings-row">
              <label>
                Datenbank-Pfad
                <input
                  bind:value={settingsForm.db_location}
                  placeholder="C:\\Users\\DeinBenutzer\\AppData\\Roaming\\Boulderado\\boulderado-pos.db"
                />
              </label>
              <small>
                Speicherort der SQLite-Datenbank. Änderungen werden beim nächsten Start übernommen.
              </small>
            </div>
            <div class="settings-row">
              <label>
                Sprache
                <select bind:value={settingsForm.language}>
                  <option value="de">Deutsch</option>
                  <option value="en">English</option>
                </select>
              </label>
              <label>
                Währung
                <select bind:value={settingsForm.currency}>
                  <option value="EUR">Euro (€)</option>
                  <option value="USD">US Dollar ($)</option>
                  <option value="CHF">Schweizer Franken (CHF)</option>
                </select>
              </label>
            </div>
            <div class="settings-row column">
              <label class="switch-row">
                <input type="checkbox" bind:checked={settingsForm.auto_updates} />
                <span>Automatische Updates & Benachrichtigungen aktivieren</span>
              </label>
              <label class="switch-row">
                <input type="checkbox" bind:checked={settingsForm.enable_backups} />
                <span>Automatische Sicherungen im Hintergrund erstellen</span>
              </label>
            </div>
            <div class="settings-row column">
              <p class="muted">
                Weitere Optionen können hier ergänzt werden (z. B. Drucker, Steuersätze oder Zahlungsanbieter).
              </p>
            </div>
            {#if settingsMessage}
              <p class={`settings-message ${settingsMessage.includes("Fehler") || settingsMessage.includes("fehlgeschlagen") ? "error" : "success"}`}>
                {settingsMessage}
              </p>
            {/if}
            <div class="form-actions">
              <button type="submit" disabled={settingsLoading}>
                {settingsLoading ? "Speichere..." : "Einstellungen speichern"}
              </button>
              <button type="button" on:click={loadSettings} disabled={settingsLoading}>
                Neu laden
              </button>
            </div>
          </form>
        {/if}
      </section>
    </div>
  </div>
{/if}

<style>
:global(body) {
  margin: 0;
  font-family: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
  background: #dcdcdc;
  color: #1d1d1d;
}

.pos-app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.pos-app.blurred-bg {
  filter: blur(8px);
  pointer-events: none;
  user-select: none;
}

.top-bar {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 1.5rem;
  background: #f2f2f2;
  border-bottom: 1px solid #cfcfcf;
}

.app-title {
  font-weight: 600;
}

.tab-strip {
  display: flex;
  gap: 0.25rem;
  flex-wrap: wrap;
}

.tab-strip button {
  border: none;
  padding: 0.35rem 0.8rem;
  border-radius: 4px;
  background: transparent;
  color: #333;
  font-weight: 500;
  cursor: pointer;
}

.tab-strip button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.tab-strip button.active {
  background: #ffffff;
  border: 1px solid #cfcfcf;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.6);
}

.status-box {
  margin-left: auto;
  background: #ffffff;
  border: 1px solid #c9c9c9;
  padding: 0.35rem 0.75rem;
  border-radius: 4px;
  text-align: right;
}

.status-box .label {
  font-size: 0.75rem;
  color: #777;
  display: block;
}

.linkish {
  border: none;
  background: none;
  color: #0077cc;
  cursor: pointer;
  padding: 0;
  margin-top: 0.3rem;
  text-decoration: underline;
}

.admin-btn {
  margin-left: 1rem;
  border: none;
  border-radius: 4px;
  padding: 0.45rem 0.8rem;
  background: #4caf50;
  color: #fff;
  cursor: pointer;
}

.nav-message {
  margin: 0.5rem 1.5rem;
  color: #c62828;
}

.nav-panel {
  padding: 1rem;
}

.content-area {
  flex: 1;
  display: grid;
  grid-template-columns: 190px minmax(0, 1fr) 240px;
  gap: 1rem;
  padding: 1rem;
}

.bucket-panel,
.receipt-panel {
  background: #f0f0f0;
  border: 1px solid #cfcfcf;
  border-radius: 6px;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.panel-title {
  font-weight: 600;
  padding: 0.4rem 0.5rem;
  background: #d8d8d8;
  border-radius: 4px;
  text-transform: uppercase;
  font-size: 0.75rem;
  color: #4a4a4a;
}

.bucket-actions {
  display: flex;
  margin-bottom: 0.5rem;
}

.create-bucket {
  flex: 1;
  border: none;
  border-radius: 4px;
  padding: 0.5rem;
  background: #b8c0c9;
  cursor: pointer;
  font-weight: 600;
  color: #1f1f1f;
}

.bucket-list {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
  overflow-y: auto;
}

.bucket-row {
  border: none;
  border-radius: 4px;
  padding: 0.4rem 0.45rem;
  background: #ffffff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
  text-align: left;
}

.bucket-row.active {
  background: #bfe1ff;
}

.bucket-row-main {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.bucket-name {
  font-weight: 600;
}

.bucket-meta {
  font-size: 0.8rem;
  color: #4c4c4c;
}

.rename-bucket {
  font-size: 0.8rem;
  color: #555;
  padding: 0.1rem 0.2rem;
}

.muted {
  color: #6f6f6f;
  font-size: 0.85rem;
  margin: 0;
}

.grid-panel {
  background: #fefefe;
  border: 1px solid #cfcfcf;
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  padding: 0.75rem;
  gap: 0.75rem;
}

.info-bar {
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 0.5rem;
  font-size: 0.9rem;
  color: #555;
}

.product-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
  gap: 0.65rem;
}

.product-card {
  border: none;
  border-radius: 6px;
  padding: 0.6rem 0.7rem;
  background: var(--accent, #fff);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  box-shadow: inset 0 -4px 0 rgba(0, 0, 0, 0.08);
  color: #1d1d1d;
  text-align: left;
}

.product-card .icon {
  font-size: 1.75rem;
}

.product-card .name {
  font-weight: 600;
  font-size: 0.95rem;
}

.product-card .note {
  font-size: 0.75rem;
  color: #212121;
}

.product-card .price {
  font-weight: 600;
  text-align: right;
}

.receipt-content {
  flex: 1;
  background: #ffffff;
  border-radius: 4px;
  border: 1px dashed #b3b3b3;
  padding: 0.75rem;
  color: #444;
  font-size: 0.9rem;
  overflow-y: auto;
}

.ticket-lines {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.ticket-line {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.ticket-line .qty {
  font-weight: 600;
  margin-right: 0.2rem;
}

.receipt-footer {
  margin-top: 0.75rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem;
  background: #fff;
  border-radius: 4px;
  border: 1px solid #dedede;
}

.receipt-footer small {
  display: block;
  color: #6a6a6a;
}

.receipt-footer .total {
  font-size: 1.2rem;
  font-weight: 700;
}

.checkout {
  margin-top: auto;
  border: none;
  border-radius: 4px;
  padding: 0.75rem;
  background: #4caf50;
  color: #fff;
  font-size: 1rem;
  cursor: pointer;
}

.checkout:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.login-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 30;
}

.login-card {
  background: #ffffff;
  border-radius: 10px;
  padding: 2rem;
  width: min(320px, 90vw);
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
}

.login-card label {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.95rem;
}

.login-card input {
  border: 1px solid #cfcfcf;
  border-radius: 5px;
  padding: 0.45rem;
  font: inherit;
}

.login-card button {
  border: none;
  border-radius: 5px;
  padding: 0.6rem;
  background: #4caf50;
  color: #fff;
  font-size: 1rem;
  cursor: pointer;
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 20;
}

.modal {
  width: min(1100px, 95vw);
  height: 90vh;
  background: #ffffff;
  border-radius: 10px;
  box-shadow: 0 15px 35px rgba(0, 0, 0, 0.35);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #ececec;
}

.close-btn {
  border: none;
  background: transparent;
  font-size: 1.5rem;
  cursor: pointer;
}

.modal-tabs {
  display: flex;
  gap: 0.25rem;
  padding: 0.75rem 1.5rem;
  border-bottom: 1px solid #ececec;
}

.modal-tabs button {
  border: none;
  padding: 0.4rem 0.8rem;
  border-radius: 4px;
  background: #f0f0f0;
  cursor: pointer;
}

.modal-tabs button.active {
  background: #bfe1ff;
  font-weight: 600;
}

.modal-body {
  padding: 1.5rem;
  overflow: auto;
  flex: 1;
}

.admin-grid {
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  gap: 1rem;
}

.admin-form {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  padding: 1rem;
  border: 1px solid #e2e2e2;
  border-radius: 6px;
  background: #fafafa;
}

.admin-form label {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  font-size: 0.9rem;
}

.admin-form input,
.admin-form textarea,
.admin-form select {
  border-radius: 4px;
  border: 1px solid #cfcfcf;
  padding: 0.4rem 0.45rem;
  font: inherit;
}

.form-actions {
  display: flex;
  gap: 0.5rem;
}

.form-actions button {
  flex: 1;
  border: none;
  border-radius: 4px;
  padding: 0.5rem;
  cursor: pointer;
  background: #4caf50;
  color: #fff;
}

.form-actions button:nth-child(2) {
  background: #d8d8d8;
  color: #333;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
}

.settings-row {
  display: flex;
  gap: 1rem;
  align-items: flex-start;
}

.settings-row.column {
  flex-direction: column;
}

.settings-row label {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  font-size: 0.9rem;
}

.settings-row small {
  color: #6f6f6f;
  font-size: 0.8rem;
}

.settings-form input,
.settings-form select,
.settings-form textarea {
  border-radius: 4px;
  border: 1px solid #cfcfcf;
  padding: 0.45rem;
  font: inherit;
}

.switch-row {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  font-weight: 500;
}

.settings-message {
  margin: 0;
  font-size: 0.9rem;
}

.settings-message.success {
  color: #2e7d32;
}

.settings-message.error {
  color: #c62828;
}

.admin-table {
  border: 1px solid #e2e2e2;
  border-radius: 6px;
  background: #fff;
  padding: 1rem;
  overflow: auto;
}

.admin-table table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

.admin-table th,
.admin-table td {
  padding: 0.5rem 0.4rem;
  border-bottom: 1px solid #f0f0f0;
  text-align: left;
}

.admin-table td.actions {
  display: flex;
  gap: 0.35rem;
}

.admin-table td.actions button {
  border: none;
  border-radius: 4px;
  padding: 0.35rem 0.5rem;
  cursor: pointer;
  background: #e1e1e1;
}

@media (max-width: 1000px) {
  .admin-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 1100px) {
  .content-area {
    grid-template-columns: 1fr;
  }

  .category-panel,
  .receipt-panel {
    order: -1;
  }
}

@media (max-width: 700px) {
  .tab-strip {
    flex-direction: column;
    width: 100%;
  }

  .product-grid {
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  }
}
</style>
