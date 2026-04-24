//! Rustok brand logo — diamond/crystal PNG.
//!
//! The PNG asset lives at `assets/rustok-logo-transparent.png` and is
//! copied to `dist/rustok-logo-transparent.png` by Trunk
//! (see `index.html`).

use leptos::prelude::*;

/// Renders the Rustok logo at a given size.
///
/// When `on_light` is `true`, the logo is wrapped in a dark rounded
/// container (for placement on light backgrounds). Otherwise, it is rendered
/// directly with a glow drop-shadow — used on dark screens like Welcome.
#[component]
pub fn RustokLogo(
    /// Logo size in pixels.
    #[prop(optional, default = 120)]
    size: u32,
    /// Wrap in a dark plate for placement on light backgrounds.
    #[prop(optional, default = false)]
    on_light: bool,
) -> impl IntoView {
    let filter = if on_light {
        "drop-shadow(0 6px 16px rgba(131,135,195,0.45))"
    } else {
        "drop-shadow(0 8px 28px rgba(131,135,195,0.55))"
    };

    let img_style = format!("display:block;width:{size}px;height:{size}px;filter:{filter};");

    if !on_light {
        return view! {
            <img
                src="rustok-logo-transparent.png"
                width=size
                height=size
                alt="Rustok"
                style=img_style
            />
        }
        .into_any();
    }

    let plate_radius = (size as f32 * 0.22) as u32;
    let inner_size = (size as f32 * 0.9) as u32;

    let plate_style = format!(
        "width:{size}px;height:{size}px;border-radius:{plate_radius}px;\
         background:radial-gradient(circle at 50% 45%, #1a2040 0%, #05070F 100%);\
         display:flex;align-items:center;justify-content:center;\
         box-shadow:0 10px 30px rgba(10,17,35,0.25);overflow:hidden;"
    );

    view! {
        <div style=plate_style>
            <img
                src="rustok-logo-transparent.png"
                width=inner_size
                height=inner_size
                alt="Rustok"
                style="display:block;"
            />
        </div>
    }
    .into_any()
}
