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
    balance_cents?: number;
  };

  type LoadMembersOptions = {
    refreshMemberships?: boolean;
  };

  type MembershipType = {
    id: number;
    name: string;
    description?: string | null;
    price_cents?: number | null;
    duration_days?: number | null;
    max_uses?: number | null;
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

  type NavItem = {
    id: "verwaltung" | "mitglieder" | "transaktionen" | "checkins";
    label: string;
    minRole: RoleName;
    mode: "view" | "modal";
  };

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

  type CheckinRecord = {
    id: number;
    member_id: number;
    member_name: string;
    membership_name?: string | null;
    created_at: string;
  };

  type MemberMembership = {
    id: number;
    member_id: number;
    membership_id: number;
    membership_name: string;
    remaining_uses?: number | null;
    start_date?: string | null;
    end_date?: string | null;
  };

  const navItems: NavItem[] = [
    { id: "verwaltung", label: "Kasse", minRole: "user", mode: "view" },
    { id: "mitglieder", label: "Mitglieder", minRole: "user", mode: "view" },
    { id: "transaktionen", label: "Transaktionen", minRole: "user", mode: "view" },
    { id: "checkins", label: "Check-ins", minRole: "manager", mode: "modal" }
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
  let memberships: MembershipType[] = [];
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
  let paymentMemberTerm = "";
  let paymentMemberResults: Member[] = [];
  let paymentMemberId: number | null = null;
  let selectedPaymentMember: Member | null = null;
  let paymentUseBalance = false;
  let checkoutMessage = "";
  let checkoutInProgress = false;

  let showAdminModal = false;
  let activeAdminTab = 0;
  let navMessage = "";
  let memberSearchTerm = "";
  let memberSearchResults: Member[] = [];
  let checkinMessage = "";
  let checkinMessageType: "success" | "error" | null = null;
  let showCheckinModal = false;
  let checkinsToday: CheckinRecord[] = [];
  let checkedInMemberIds = new Set<number>();

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
    notes: "",
    balance_cents: 0
  });
  let memberForm = blankMemberForm();

  const blankMembershipForm = () => ({
    id: null as number | null,
    name: "",
    description: "",
    price_cents: 0,
    duration_days: null as number | null,
    max_uses: null as number | null
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
  let memberMemberships: MemberMembership[] = [];
  let newMemberMembershipId: number | null = null;
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
      notes: "Testzugang",
      active_membership_id: 1,
      active_membership_type: "Demo Mitgliedschaft",
      balance_cents: 1500
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
  const mockMemberships: MembershipType[] = [
    {
      id: 6001,
      name: "Demo Mitgliedschaft",
      description: "Unbegrenzter Zugang",
      price_cents: 1500,
      duration_days: 30,
      max_uses: null
    }
  ];
  const mockCheckins: CheckinRecord[] = [
    {
      id: 1,
      member_id: 8001,
      member_name: "Demo Mitglied",
      membership_name: "Demo Mitgliedschaft",
      created_at: "2024-01-01T11:15:00Z"
    }
  ];
  const mockMemberMemberships: MemberMembership[] = [
    {
      id: 5001,
      member_id: 8001,
      membership_id: 6001,
      membership_name: "Demo Mitgliedschaft",
      remaining_uses: 3,
      start_date: "2024-01-01",
      end_date: "2024-01-31"
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
  $: displayMemberships = currentUser ? memberships : mockMemberships;
  $: displayTransactionsToday = currentUser ? transactionsToday : mockTransactionsToday;
  $: sourceMemberMemberships = currentUser ? memberMemberships : mockMemberMemberships;
  $: memberMembershipMap = sourceMemberMemberships.reduce((map, mm) => {
      const list = map.get(mm.member_id) ?? [];
      list.push(mm);
      map.set(mm.member_id, list);
      return map;
    }, new Map<number, MemberMembership[]>());
  const formatMembershipInstance = (mm: MemberMembership) => {
    if (!mm) return "";
    const parts = [] as string[];
    if (mm.remaining_uses != null) {
      parts.push(`${mm.remaining_uses}x`);
    }
    if (mm.end_date) {
      parts.push(`bis ${mm.end_date}`);
    }
    const details = parts.length ? ` (${parts.join(", ")})` : "";
    return `${mm.membership_name}${details}`;
  };
  $: memberMembershipSummary = (() => {
    const map = new Map<number, string[]>();
    memberMembershipMap.forEach((list, memberId) => {
      map.set(memberId, list.map(formatMembershipInstance));
    });
    return map;
  })();
  const membershipSummaryText = (memberId: number) => {
    const list = memberMembershipSummary.get(memberId) ?? [];
    return list.length ? list.join(", ") : "Keine Mitgliedschaft";
  };
  $: selectedPaymentMember =
    paymentMemberId != null
      ? displayMembers.find((member) => member.id === paymentMemberId) ?? null
      : null;
  $: paymentMemberResults = (() => {
    const trimmed = paymentMemberTerm.trim().toLowerCase();
    if (!trimmed) {
      return [];
    }
    if (
      selectedPaymentMember &&
      `${selectedPaymentMember.first_name} ${selectedPaymentMember.last_name}`
        .trim()
        .toLowerCase() === trimmed
    ) {
      return [];
    }
    return displayMembers
      .filter((member) =>
        `${member.first_name} ${member.last_name} ${member.email ?? ""}`
          .toLowerCase()
          .includes(trimmed)
      )
      .slice(0, 5);
  })();
  $: if (!selectedPaymentMember) {
    paymentUseBalance = false;
  }
  $: displayCheckins = currentUser ? checkinsToday : mockCheckins;
  $: checkedInMemberIds = new Set(displayCheckins.map((entry) => entry.member_id));
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
    await createBucketWithName(null);
  }

  async function handleBucketSelect(bucketId: number) {
    activeBucketId = bucketId;
    await loadBucketItems(bucketId);
  }

  async function handleCloseBucket(bucketId: number) {
    if (!currentUser) return;
    if (!window.confirm("Bucket abschließen und entfernen?")) return;
    try {
      await invoke("delete_bucket", { payload: { bucketId } });
      if (activeBucketId === bucketId) {
        activeBucketId = null;
      }
      await loadBuckets();
      if (activeBucketId) {
        await loadBucketItems(activeBucketId);
      } else {
        bucketItems = [];
      }
    } catch (error) {
      console.error("Bucket konnte nicht gelöscht werden", error);
    }
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

  function activateOnEnterOrSpace(event: KeyboardEvent, callback: () => void) {
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      callback();
    }
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

  function handlePaymentMemberInput(event: Event) {
    const value = (event.currentTarget as HTMLInputElement)?.value ?? "";
    if (
      paymentMemberId &&
      selectedPaymentMember &&
      value.trim().toLowerCase() !==
        `${selectedPaymentMember.first_name} ${selectedPaymentMember.last_name}`
          .trim()
          .toLowerCase()
    ) {
      paymentMemberId = null;
    }
    paymentMemberTerm = value;
  }

  function selectPaymentMember(member: Member) {
    paymentMemberId = member.id;
    paymentMemberTerm = `${member.first_name} ${member.last_name}`.trim();
  }

  function clearPaymentMember() {
    paymentMemberId = null;
    paymentMemberTerm = "";
  }

  async function startCheckout() {
    if (
      !currentUser ||
      !activeBucketId ||
      bucketItems.length === 0 ||
      bucketItemsLoading ||
      checkoutInProgress
    ) {
      return;
    }
    if (paymentUseBalance && !selectedPaymentMember) {
      checkoutMessage = "Mitglied für Guthaben-Zahlung auswählen.";
      return;
    }
    checkoutInProgress = true;
    checkoutMessage = "";
    try {
      await invoke("checkout_bucket", {
        payload: {
          bucketId: activeBucketId,
          memberId: selectedPaymentMember?.id ?? null,
          useBalance: paymentUseBalance,
          paymentMethod: paymentUseBalance ? "Guthaben" : "Bar"
        }
      });
      checkoutMessage = "Zahlung abgeschlossen.";
      paymentMemberId = null;
      paymentMemberTerm = "";
      paymentUseBalance = false;
      await loadBuckets();
      if (activeBucketId) {
        await loadBucketItems(activeBucketId);
      } else {
        bucketItems = [];
      }
      await Promise.all([loadMembers({ refreshMemberships: true }), loadTransactionsToday()]);
    } catch (error) {
      console.error("Checkout fehlgeschlagen", error);
      checkoutMessage = "Zahlung fehlgeschlagen.";
    } finally {
      checkoutInProgress = false;
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
        await loadMembers({ refreshMemberships: true });
        break;
      case "Mitgliedschaften":
        await Promise.all([loadMembers({ refreshMemberships: true }), loadMemberships()]);
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

  async function loadMembers(options?: LoadMembersOptions) {
    try {
      const data = await invoke<Member[]>("list_members");
      members =
        data?.map((member: any) => ({
          id: member.id,
          first_name: member.first_name ?? member.firstName ?? "",
          last_name: member.last_name ?? member.lastName ?? "",
          email: member.email ?? null,
          phone: member.phone ?? null,
          status: member.status ?? "active",
          notes: member.notes ?? "",
          balance_cents: member.balance_cents ?? member.balanceCents ?? 0
        })) ?? [];
      if (options?.refreshMemberships) {
        await loadMemberMemberships();
      }
    } catch (error) {
      console.error("Mitglieder konnten nicht geladen werden", error);
    }
  }

  async function submitMember() {
    if (!memberForm.first_name.trim() || !memberForm.last_name.trim()) return;
    try {
      const payload = {
        id: memberForm.id,
        firstName: memberForm.first_name,
        lastName: memberForm.last_name,
        email: memberForm.email,
        phone: memberForm.phone,
        status: memberForm.status,
        notes: memberForm.notes,
        activeMembershipId: memberForm.active_membership_id,
        balanceCents: memberForm.balance_cents
      };
      await invoke("save_member", { payload });
      memberForm = blankMemberForm();
      await loadMembers({ refreshMemberships: true });
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
      notes: member.notes ?? "",
      balance_cents: member.balance_cents ?? 0
    };
  }

  async function removeMember(id: number) {
    if (!window.confirm("Mitglied löschen? Zugehörige Mitgliedschaften werden ebenfalls gelöscht.")) return;
    try {
      await invoke("delete_member", { id });
      if (memberForm.id === id) {
        memberForm = blankMemberForm();
      }
      await Promise.all([loadMembers({ refreshMemberships: true }), loadMemberships()]);
    } catch (error) {
      console.error("Mitglied konnte nicht gelöscht werden", error);
    }
  }

  async function loadMemberships() {
    try {
      const data = await invoke<MembershipType[]>("list_memberships");
      memberships =
        data?.map((ms: any) => ({
          id: ms.id,
          name: ms.name ?? ms.membership_type ?? "",
          description: ms.description ?? ms.notes ?? "",
          price_cents: ms.price_cents ?? ms.priceCents ?? 0,
          duration_days: ms.duration_days ?? ms.durationDays ?? null,
          max_uses: ms.max_uses ?? ms.maxUses ?? null
        })) ?? [];
    } catch (error) {
      console.error("Mitgliedschaften konnten nicht geladen werden", error);
    }
  }

  async function loadMemberMemberships() {
    try {
      const data = await invoke<MemberMembershipRecord[]>("list_member_memberships");
      memberMemberships =
        data?.map((mm: any) => ({
          id: mm.id,
          member_id: mm.member_id ?? mm.memberId,
          membership_id: mm.membership_id ?? mm.membershipId,
          membership_name: mm.membership_name ?? mm.membershipName ?? "",
          remaining_uses: mm.remaining_uses ?? mm.remainingUses ?? null,
          start_date: mm.start_date ?? mm.startDate ?? null,
          end_date: mm.end_date ?? mm.endDate ?? null,
          created_at: mm.created_at ?? mm.createdAt ?? ""
        })) ?? [];
    } catch (error) {
      console.error("Mitgliedschaftszuweisungen konnten nicht geladen werden", error);
    }
  }

  async function submitMembership() {
    if (!membershipForm.name.trim()) return;
    try {
      const payload = {
        id: membershipForm.id,
        name: membershipForm.name,
        description: membershipForm.description || null,
        priceCents: membershipForm.price_cents ?? 0,
        durationDays: membershipForm.duration_days,
        maxUses: membershipForm.max_uses
      };
      await invoke("save_membership", { payload });
      membershipForm = blankMembershipForm();
      await loadMemberships();
    } catch (error) {
      console.error("Mitgliedschaft konnte nicht gespeichert werden", error);
    }
  }

  function editMembership(ms: MembershipType) {
    membershipForm = {
      id: ms.id,
      name: ms.name,
      description: ms.description ?? "",
      price_cents: ms.price_cents ?? 0,
      duration_days: ms.duration_days ?? null,
      max_uses: ms.max_uses ?? null
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
      await loadMemberMemberships();
    } catch (error) {
      console.error("Mitgliedschaft konnte nicht gelöscht werden", error);
    }
  }

  async function assignMembershipToMember() {
    if (!memberForm.id || !newMemberMembershipId) {
      return;
    }
    try {
      await invoke("assign_member_membership", {
        payload: { memberId: memberForm.id, membershipId: newMemberMembershipId }
      });
      newMemberMembershipId = null;
      await loadMembers({ refreshMemberships: true });
    } catch (error) {
      console.error("Mitgliedschaft konnte nicht zugewiesen werden", error);
    }
  }

  async function removeMemberMembership(id: number) {
    if (!window.confirm("Zuweisung entfernen?")) return;
    try {
      await invoke("delete_member_membership", { id });
      await loadMembers({ refreshMemberships: true });
    } catch (error) {
      console.error("Zuweisung konnte nicht gelöscht werden", error);
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

  async function loadCheckinsToday() {
    try {
      const data = await invoke<CheckinRecord[]>("list_checkins_today");
      checkinsToday =
        data?.map((entry: any) => ({
          id: entry.id,
          member_id: entry.member_id ?? entry.memberId,
          member_name: entry.member_name ?? entry.memberName ?? "",
          membership_name: entry.membership_name ?? entry.membershipName ?? null,
          created_at: entry.created_at ?? entry.createdAt ?? ""
        })) ?? [];
    } catch (error) {
      console.error("Check-ins konnten nicht geladen werden", error);
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

  function canAccessNav(itemId: NavItem["id"]) {
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
      await Promise.all([loadMembers({ refreshMemberships: true }), loadMemberships()]);
    } else if (itemId === "transaktionen") {
      await loadTransactionsToday();
    }
  }

  async function handleNavClick(item: NavItem) {
    navMessage = "";
    if (!canAccessNav(item.id)) {
      navMessage = "Keine Berechtigung für diesen Bereich.";
      return;
    }
    if (item.mode === "modal") {
      await openCheckinModal();
      return;
    }
    await handleNavChange(item.id as "verwaltung" | "mitglieder" | "transaktionen");
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
      checkoutMessage = "";
      paymentMemberTerm = "";
      paymentMemberId = null;
      paymentUseBalance = false;
      if (!canAccessNav(activeNav)) {
        activeNav = currentUser?.role === "user" ? "mitglieder" : "verwaltung";
      }
      await initializeBuckets();
      await loadMemberMemberships();
      await Promise.all([
        loadMembers(),
        loadMemberships(),
        loadTransactionsToday(),
        loadCheckinsToday()
      ]);
    } catch (error) {
      console.error("Login fehlgeschlagen", error);
      loginError = "Anmeldung nicht möglich.";
    }
  }

  function logout() {
    currentUser = null;
    showAdminModal = false;
    activeNav = "verwaltung";
    memberMemberships = [];
    checkinsToday = [];
    memberSearchTerm = "";
    checkinMessage = "";
    checkinMessageType = null;
    paymentMemberTerm = "";
    paymentMemberId = null;
    paymentUseBalance = false;
    checkoutMessage = "";
    checkoutInProgress = false;
  }

  $: if (currentUser && !canAccessNav(activeNav)) {
    activeNav = currentUser.role === "user" ? "mitglieder" : "verwaltung";
  }

  async function openCheckinModal() {
    if (!currentUser) return;
    await loadCheckinsToday();
    showCheckinModal = true;
  }

  function closeCheckinModal() {
    showCheckinModal = false;
  }

  async function checkInMember(member: Member) {
    if (!currentUser) return;
    if (checkedInMemberIds.has(member.id)) {
      checkinMessage = `${member.first_name} ${member.last_name} ist bereits eingecheckt.`;
      checkinMessageType = "error";
      return;
    }
    try {
      checkinMessage = "";
      checkinMessageType = null;
      await invoke("record_checkin", { payload: { memberId: member.id } });
      checkinMessage = `${member.first_name} ${member.last_name} eingecheckt.`;
      checkinMessageType = "success";
      memberSearchTerm = "";
      await Promise.all([loadCheckinsToday(), loadMemberMemberships()]);
    } catch (error) {
      console.error("Check-in fehlgeschlagen", error);
      const message =
        typeof error === "string"
          ? error
          : (error as { message?: string })?.message ?? "Check-in fehlgeschlagen.";
      checkinMessage = message;
      checkinMessageType = "error";
    }
  }

  async function deleteCheckinRecord(id: number) {
    if (!currentUser) return;
    if (!window.confirm("Check-in löschen?")) return;
    try {
      await invoke("delete_checkin", { id });
      await Promise.all([loadCheckinsToday(), loadMemberMemberships()]);
    } catch (error) {
      console.error("Check-in konnte nicht gelöscht werden", error);
    }
  }

  $: memberSearchResults = memberSearchTerm.trim()
    ? displayMembers
        .filter((member) =>
          `${member.first_name} ${member.last_name} ${member.email ?? ""} ${
            membershipSummaryText(member.id)
          }`
            .toLowerCase()
            .includes(memberSearchTerm.trim().toLowerCase())
        )
        .slice(0, 5)
    : displayMembers.slice(0, 5);
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
          class:active={item.mode === "view" && activeNav === item.id}
          disabled={!currentUser || !canAccessNav(item.id)}
          on:click={() => handleNavClick(item)}
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
            <div
              role="button"
              tabindex="0"
              aria-pressed={bucket.id === displayActiveBucketId}
              class="bucket-row"
              class:active={bucket.id === displayActiveBucketId}
              on:click={() => currentUser && handleBucketSelect(bucket.id)}
              on:keydown={(event) => currentUser && activateOnEnterOrSpace(event, () => handleBucketSelect(bucket.id))}
            >
              <div class="bucket-row-main">
                <span class="bucket-name">{bucket.name}</span>
                <span class="bucket-meta">
                  {bucket.item_count} · {formatPrice(bucket.total_cents)}
                </span>
              </div>
              {#if currentUser}
                <div class="bucket-row-actions">
                  <button
                    type="button"
                    class="bucket-icon danger"
                    title="Abschließen & entfernen"
                    on:click|stopPropagation={() => handleCloseBucket(bucket.id)}
                  >
                    🗑
                  </button>
                </div>
              {/if}
            </div>
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
      <div class="payment-section">
        <label>
          Mitglied (optional)
          <input
            type="text"
            placeholder="Name oder E-Mail"
            value={paymentMemberTerm}
            on:input={handlePaymentMemberInput}
            disabled={!currentUser}
          />
        </label>
        {#if currentUser && paymentMemberResults.length > 0}
          <div class="payment-results">
            {#each paymentMemberResults as member}
              <button type="button" on:click={() => selectPaymentMember(member)}>
                <span>{member.first_name} {member.last_name}</span>
                <small>Guthaben: {formatPrice(member.balance_cents ?? 0)}</small>
              </button>
            {/each}
          </div>
        {/if}
        {#if selectedPaymentMember}
          <div class="selected-member">
            <div>
              <strong>{selectedPaymentMember.first_name} {selectedPaymentMember.last_name}</strong>
              <small>Guthaben: {formatPrice(selectedPaymentMember.balance_cents ?? 0)}</small>
            </div>
            <button type="button" class="linkish" on:click={clearPaymentMember}>Entfernen</button>
          </div>
          <label class="balance-option">
            <input
              type="checkbox"
              bind:checked={paymentUseBalance}
              disabled={!currentUser}
            />
            Mit Guthaben bezahlen ({formatPrice(bucketTotal)})
          </label>
        {/if}
        {#if checkoutMessage}
          <p class="status-note">{checkoutMessage}</p>
        {/if}
      </div>
      <button
        class="checkout"
        type="button"
        on:click={startCheckout}
        disabled={
          !currentUser ||
          !activeBucketId ||
          bucketItems.length === 0 ||
          bucketItemsLoading ||
          checkoutInProgress
        }
      >
        {checkoutInProgress ? "Verarbeite..." : "Zahlung starten"}
      </button>
    </aside>
    </section>
    <section class="checkin-panel">
      <h3>Mitglied Check-in</h3>
      <p>Mitglieder schnell suchen und den Besuch erfassen.</p>
      <input
        type="text"
        placeholder="Name oder E-Mail suchen"
        bind:value={memberSearchTerm}
        disabled={!currentUser}
      />
      {#if checkinMessage}
        <p class="status-note" class:error={checkinMessageType === "error"}>
          {checkinMessage}
        </p>
      {/if}
      <div class="checkin-results">
        {#if !currentUser}
          <p>Bitte anmelden, um Check-ins zu erfassen.</p>
        {:else if memberSearchResults.length === 0}
          <p>Keine Mitglieder gefunden.</p>
        {:else}
          {#each memberSearchResults as member}
            <div class="checkin-result">
              <div>
                <strong>{member.first_name} {member.last_name}</strong>
                <small>{membershipSummaryText(member.id)}</small>
                <small>Guthaben: {formatPrice(member.balance_cents ?? 0)}</small>
              </div>
              <div class="checkin-actions">
                {#if checkedInMemberIds.has(member.id)}
                  <small class="checked-in-flag">Bereits eingecheckt</small>
                {/if}
                <button
                  type="button"
                  on:click={() => checkInMember(member)}
                  disabled={!currentUser || checkedInMemberIds.has(member.id)}
                >
                  Einchecken
                </button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
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
            Guthaben (€)
            <input
              type="number"
              step="0.01"
              value={(memberForm.balance_cents ?? 0) / 100}
              on:input={(event) =>
                (memberForm.balance_cents = euroInputToCents(
                  (event.currentTarget as HTMLInputElement).value
                ))}
            />
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
                <th>Mitgliedschaften</th>
                <th>Guthaben</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              {#each displayMembers as member}
                <tr>
                  <td>{member.first_name} {member.last_name}</td>
                  <td>{member.email ?? "—"}</td>
                  <td>{member.status}</td>
                  <td>{membershipSummaryText(member.id)}</td>
                  <td>{formatPrice(member.balance_cents ?? 0)}</td>
                  <td class="actions">
                    <button type="button" on:click={() => editMember(member)}>Bearbeiten</button>
                    <button type="button" on:click={() => removeMember(member.id)}>Löschen</button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
        {#if memberForm.id}
          <div class="member-memberships">
            <h4>Zugewiesene Mitgliedschaften</h4>
            <div class="assign-row">
              <select bind:value={newMemberMembershipId}>
                <option value="">Typ wählen</option>
                {#each displayMemberships as membership}
                  <option value={membership.id}>{membership.name}</option>
                {/each}
              </select>
              <button
                type="button"
                on:click={assignMembershipToMember}
                disabled={!newMemberMembershipId}
              >
                Zuweisen
              </button>
            </div>
            <div class="admin-table compact">
              <table>
                <thead>
                  <tr>
                    <th>Mitgliedschaft</th>
                    <th>Start</th>
                    <th>Ende</th>
                    <th>Verbleibend</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each memberMembershipMap.get(memberForm.id) ?? [] as assignment}
                    <tr>
                      <td>{formatMembershipInstance(assignment)}</td>
                      <td>{assignment.start_date ?? "—"}</td>
                      <td>{assignment.end_date ?? "—"}</td>
                      <td>
                        {assignment.remaining_uses ?? "∞"}
                      </td>
                      <td class="actions">
                        <button type="button" on:click={() => removeMemberMembership(assignment.id)}>Entfernen</button>
                      </td>
                    </tr>
                  {/each}
                  {#if (memberMembershipMap.get(memberForm.id) ?? []).length === 0}
                    <tr>
                      <td colspan="5">Keine aktiven Mitgliedschaften.</td>
                    </tr>
                  {/if}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
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
  <div
    class="modal-overlay"
    role="button"
    tabindex="0"
    aria-label="Verwaltungsmenü schließen"
    on:click={closeAdminModal}
    on:keydown={(event) => activateOnEnterOrSpace(event, closeAdminModal)}
  >
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="admin-modal-title" on:click|stopPropagation>
      <header class="modal-header">
        <h2 id="admin-modal-title">Verwaltung</h2>
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
                Guthaben (€)
                <input
                  type="number"
                  step="0.01"
                  value={(memberForm.balance_cents ?? 0) / 100}
                  on:input={(event) =>
                    (memberForm.balance_cents = euroInputToCents(
                      (event.currentTarget as HTMLInputElement).value
                    ))}
                />
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
                    <th>Mitgliedschaft</th>
                    <th>Guthaben</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each members as member}
                    <tr>
                      <td>{member.first_name} {member.last_name}</td>
                      <td>{member.email ?? "—"}</td>
                      <td>{member.status}</td>
                      <td>{member.active_membership_type ?? "—"}</td>
                      <td>{formatPrice(member.balance_cents ?? 0)}</td>
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
                Name
                <input bind:value={membershipForm.name} required />
              </label>
              <label>
                Beschreibung
                <textarea rows="2" bind:value={membershipForm.description}></textarea>
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
                Dauer (Tage)
                <input
                  type="number"
                  min="0"
                  value={membershipForm.duration_days ?? ""}
                  on:input={(event) => {
                    const value = (event.currentTarget as HTMLInputElement).value;
                    membershipForm.duration_days = value ? Number(value) : null;
                  }}
                />
              </label>
              <label>
                Maximale Nutzungen
                <input
                  type="number"
                  min="0"
                  value={membershipForm.max_uses ?? ""}
                  on:input={(event) => {
                    const value = (event.currentTarget as HTMLInputElement).value;
                    membershipForm.max_uses = value ? Number(value) : null;
                  }}
                />
              </label>
              <div class="form-actions">
                <button type="submit">{membershipForm.id ? "Aktualisieren" : "Speichern"}</button>
                <button type="button" on:click={() => (membershipForm = blankMembershipForm())}>Zurücksetzen</button>
              </div>
            </form>
            <div class="admin-table">
              <h3>Mitgliedschaftstypen</h3>
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Preis</th>
                    <th>Dauer (Tage)</th>
                    <th>Nutzungen</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  {#each memberships as ms}
                    <tr>
                      <td>{ms.name}</td>
                      <td>{formatPrice(ms.price_cents ?? 0)}</td>
                      <td>{ms.duration_days ?? "—"}</td>
                      <td>{ms.max_uses ?? "unbegrenzt"}</td>
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

{#if showCheckinModal}
  <div
    class="modal-overlay"
    role="button"
    tabindex="0"
    aria-label="Check-in-Übersicht schließen"
    on:click={closeCheckinModal}
    on:keydown={(event) => activateOnEnterOrSpace(event, closeCheckinModal)}
  >
    <div
      class="modal checkins-modal"
      role="dialog"
      aria-modal="true"
      aria-labelledby="checkin-modal-title"
      on:click|stopPropagation
    >
      <header class="modal-header">
        <h2 id="checkin-modal-title">Heutige Check-ins</h2>
        <button class="close-btn" type="button" on:click={closeCheckinModal}>×</button>
      </header>
      <section class="modal-body">
        <div class="admin-table">
          <table>
            <thead>
              <tr>
                <th>Mitglied</th>
                <th>Mitgliedschaft</th>
                <th>Zeit</th>
                <th>Aktion</th>
              </tr>
            </thead>
            <tbody>
              {#if displayCheckins.length === 0}
                <tr>
                  <td colspan="4">Keine Check-ins heute.</td>
                </tr>
              {:else}
                {#each displayCheckins as checkin}
                  <tr>
                    <td>{checkin.member_name}</td>
                    <td>{checkin.membership_name ?? "Keine"}</td>
                    <td>{new Date(checkin.created_at).toLocaleTimeString("de-DE")}</td>
                    <td>
                      <button
                        type="button"
                        class="checkin-delete-btn"
                        on:click={() => deleteCheckinRecord(checkin.id)}
                        disabled={!currentUser}
                      >
                        Entfernen
                      </button>
                    </td>
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
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

.checkin-panel {
  margin: 1rem;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.checkin-panel input {
  border-radius: 4px;
  border: 1px solid #cfcfcf;
  padding: 0.5rem;
  font: inherit;
}

.status-note {
  margin: 0;
  color: #2e7d32;
  font-size: 0.9rem;
}

.status-note.error {
  color: #b71c1c;
}

.checkin-results {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.checkin-result {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.6rem;
  background: #ffffff;
  border-radius: 6px;
  border: 1px solid #e0e0e0;
}

.checkin-result strong {
  display: block;
}

.checkin-result small {
  display: block;
  color: #666;
}

.checkin-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 0.25rem;
}

.checkin-result button {
  border: none;
  border-radius: 4px;
  padding: 0.4rem 0.8rem;
  background: #4caf50;
  color: #fff;
  cursor: pointer;
}

.checkin-result button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.checked-in-flag {
  font-size: 0.75rem;
  color: #b71c1c;
  font-weight: 600;
}

.checkin-delete-btn {
  border: none;
  border-radius: 4px;
  padding: 0.3rem 0.6rem;
  background: #d32f2f;
  color: #fff;
  cursor: pointer;
}

.checkin-delete-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.checkins-modal .admin-table {
  max-height: 60vh;
  overflow: auto;
}

.checkins-modal {
  width: min(700px, 90vw);
  height: auto;
}

.member-memberships {
  margin-top: 1rem;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 0.8rem;
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
}

.member-memberships h4 {
  margin: 0;
}

.assign-row {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.assign-row select {
  flex: 1;
  border: 1px solid #cfcfcf;
  border-radius: 4px;
  padding: 0.4rem;
}

.assign-row button,
.member-memberships .actions button {
  border: none;
  border-radius: 4px;
  padding: 0.4rem 0.8rem;
  background: #1976d2;
  color: #fff;
  cursor: pointer;
}

.assign-row button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.member-memberships .admin-table {
  padding: 0.5rem;
}

.member-memberships table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85rem;
}

.member-memberships table th,
.member-memberships table td {
  padding: 0.3rem 0.4rem;
  border-bottom: 1px solid #f0f0f0;
}

.member-memberships table tr:last-child td {
  border-bottom: none;
}

.member-memberships table td:nth-child(4) {
  text-align: center;
}

.member-memberships table td:last-child {
  text-align: right;
}

.member-memberships table .actions button {
  background: #d32f2f;
  padding: 0.25rem 0.6rem;
}

.member-memberships table td[colspan="5"] {
  text-align: center;
  color: #666;
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
  border: 1px solid transparent;
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

.bucket-row:hover {
  border-color: #bed6ff;
}

.bucket-row.active {
  background: #bfe1ff;
  border-color: #6fa9f7;
}

.bucket-row-main {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}

.bucket-row-actions {
  display: flex;
  gap: 0.25rem;
}

.bucket-icon {
  border: none;
  border-radius: 4px;
  padding: 0.15rem 0.35rem;
  background: #e7e7e7;
  cursor: pointer;
  font-size: 0.85rem;
}

.bucket-icon:hover {
  background: #d0d0d0;
}

.bucket-icon.danger {
  background: #fde7e7;
  color: #b71c1c;
}

.bucket-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.bucket-name {
  font-weight: 600;
}

.bucket-meta {
  font-size: 0.8rem;
  color: #4c4c4c;
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

.payment-section {
  border-top: 1px solid #d5d5d5;
  padding-top: 0.5rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.payment-section label {
  font-size: 0.85rem;
  color: #4a4a4a;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.payment-section input {
  border: 1px solid #cfcfcf;
  border-radius: 4px;
  padding: 0.4rem;
  font: inherit;
  background: #fff;
}

.payment-results {
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
}

.payment-results button {
  border: 1px solid #dedede;
  border-radius: 4px;
  background: #ffffff;
  padding: 0.35rem 0.5rem;
  text-align: left;
  cursor: pointer;
  display: flex;
  flex-direction: column;
}

.payment-results button:hover {
  background: #f2f2f2;
}

.payment-results small {
  font-size: 0.75rem;
  color: #666;
}

.selected-member {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border: 1px solid #dedede;
  border-radius: 4px;
  padding: 0.4rem 0.5rem;
  background: #ffffff;
  font-size: 0.85rem;
  gap: 0.6rem;
}

.balance-option {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.85rem;
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
