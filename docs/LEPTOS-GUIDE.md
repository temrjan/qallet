# Leptos 0.7 — Guide for Rustok

> Справочник по Leptos patterns для проекта. Составлен на основе research (2026-04-05).
> Leptos 0.8.17 доступен — миграция тривиальна для CSR. Пока работаем на 0.7.

---

## Signals

```rust
// Основной паттерн (0.7+)
let (count, set_count) = signal(0);

// Чтение
count.get()       // клонирует значение
count.read()      // borrowит без клонирования (0.7+, лучше для Vec/String)

// Запись
set_count.set(5);
set_count.update(|n| *n += 1);
set_names.write().push("Alice".to_string());  // мутабельный guard

// Derived signal — просто closure
let double = move || count.get() * 2;

// Memo — кэшированное вычисление
let memoized = Memo::new(move |_| count.get() * 2);
```

---

## Components

```rust
use leptos::prelude::*;

// Базовый
#[component]
pub fn Card() -> impl IntoView {
    view! { <div>"Hello"</div> }
}

// С props
#[component]
pub fn Card(title: String, #[prop(optional)] subtitle: Option<String>) -> impl IntoView {
    view! {
        <h2>{title}</h2>
        {subtitle.map(|s| view! { <p>{s}</p> })}
    }
}

// С children
#[component]
pub fn Wrapper(children: Children) -> impl IntoView {
    view! { <div class="wrapper">{children()}</div> }
}
```

---

## Router (leptos_router 0.7)

```rust
use leptos_router::components::*;  // Router, Routes, Route, A, ParentRoute, Outlet
use leptos_router::path;           // path!() macro

// Определение маршрутов
<Router>
    <Routes fallback=|| view! { <p>"Not found"</p> }>
        <Route path=path!("/") view=HomePage />
        <Route path=path!("/about") view=AboutPage />
    </Routes>
</Router>

// Навигация — использовать <A>, НЕ <a>
// <A> делает client-side navigation без перезагрузки + aria-current
<A href="/" attr:class="hover:text-blue-400">"Home"</A>

// Программная навигация
use leptos_router::hooks::use_navigate;
let navigate = use_navigate();
navigate("/wallet", Default::default());

// Вложенные маршруты
<ParentRoute path=path!("/settings") view=SettingsLayout>
    <Route path=path!("/profile") view=Profile />
</ParentRoute>
// Parent рендерит <Outlet /> где должны появиться children
```

**Важно:** `leptos_router = "0.7"` без feature `csr`. Feature `csr` есть только у `leptos`.

---

## Async в компонентах

### spawn_local (что мы используем)
```rust
use leptos::task::spawn_local;

let fetch = move |_| {
    spawn_local(async move {
        let result = tauri_invoke::<_, T>("cmd", &args).await;
        // update signals
    });
};
```

### Action (лучше для button-triggered async)
```rust
let fetch_balance = Action::new(move |addr: &String| {
    let addr = addr.clone();
    async move {
        tauri_invoke::<_, UnifiedBalance>("get_balance", &BalanceArgs { address: addr }).await
    }
});

// В view:
<button on:click=move |_| fetch_balance.dispatch(address.get())>"Check"</button>
{move || fetch_balance.pending().get().then(|| view! { <p>"Loading..."</p> })}
{move || fetch_balance.value().get().map(|result| match result {
    Ok(b) => view! { <p>{b.approximate_total_formatted}</p> }.into_any(),
    Err(e) => view! { <p class="text-red-400">{e}</p> }.into_any(),
})}
```

### LocalResource (для автозагрузки при изменении signal)
```rust
let balance = LocalResource::new(move || {
    let addr = address.get();
    async move {
        if addr.is_empty() { return None; }
        tauri_invoke::<_, T>("get_balance", &BalanceArgs { address: addr }).await.ok()
    }
});
```

| Паттерн | Когда |
|---------|-------|
| `spawn_local` + signals | Простые случаи, полный контроль |
| `Action` | User-triggered (кнопки). Даёт `.pending()`, `.value()` бесплатно |
| `LocalResource` | Авто-загрузка при изменении зависимости |
| `Resource` | SSR-совместимая загрузка (не нужно для Tauri CSR) |

---

## Tauri Bridge

Наш паттерн — стандартный и правильный:

```rust
// app/src/src/bridge.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

pub async fn tauri_invoke<A, R>(cmd: &str, args: &A) -> Result<R, String>
where
    A: Serialize,
    R: for<'de> Deserialize<'de>,
{ ... }
```

**Обязательно:** `"withGlobalTauri": true` в `tauri.conf.json`.

Альтернативные крейты (не переключаемся, но для информации):
- `tauri-wasm` — higher-level абстракция над invoke
- `tauri-interop` — generated typed bindings

---

## Error Handling

Для Tauri CSR приложения — ручные error signals проще чем `ErrorBoundary`.
`ErrorBoundary` лучше работает с SSR/server functions.

```rust
let (error, set_error) = signal(None::<String>);

// В async:
match tauri_invoke(...).await {
    Ok(data) => set_data.set(Some(data)),
    Err(e) => set_error.set(Some(e)),
}

// В view:
{move || error.get().map(|e| view! { <p class="text-red-400">{e}</p> })}
```

---

## WASM Size

- 3 MB dev build — **нормально**
- Для Tauri desktop размер не критичен (грузится с диска)
- Release оптимизация:

```toml
# app/src/Cargo.toml
[profile.release]
opt-level = 'z'
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

- `wasm-opt` (из binaryen) может дополнительно уменьшить
- Избегать `regex` crate (+500kb)

---

## Миграция 0.7 → 0.8

Breaking changes минимальны для CSR:
1. `LocalResource` больше не оборачивает в `SendWrapper` (убрать `.as_deref()`)
2. Server function error types изменились (не касается CSR)
3. Axum 0.8 support (не касается Tauri)

Рекомендация: бампнуть на 0.8 после стабилизации Phase 2 scaffold.
Все текущие docs/book таргетят 0.8.

---

## CSR Feature

`leptos = { version = "0.7", features = ["csr"] }` — правильно.
Ровно одна из: `csr`, `hydrate`, `ssr`. Для Tauri + Trunk = `csr`.
