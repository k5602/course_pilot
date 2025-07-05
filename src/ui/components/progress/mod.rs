use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    #[props(optional, default = 0.0)]
    pub value: f64,
    #[props(optional, default = 100.0)]
    pub max: f64,
    #[props(optional)]
    pub class: Option<String>,
    #[props(optional)]
    pub aria_label: Option<String>,
}

/// Custom Progress component using native <progress> and CSS
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    rsx! {
        progress {
            class: props.class.clone().unwrap_or_else(|| "progress".to_string()),
            value: props.value,
            max: props.max,
            aria_label: props.aria_label.clone().unwrap_or("Progressbar".to_string())
            // Optionally, you can add children or a custom indicator here if needed
        }
    }
}

/// Demo for the Progress component
#[component]
pub(super) fn Demo() -> Element {
    let mut progress = use_signal(|| 0.0);

    use_effect(move || {
        let mut timer = document::eval(
            "setInterval(() => {
                dioxus.send(Math.floor(Math.random() * 30));
            }, 1000);",
        );
        spawn(async move {
            while let Ok(new_progress) = timer.recv::<usize>().await {
                let mut progress = progress.write();
                *progress = (*progress + new_progress as f64) % 101.0;
            }
        });
    });

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/progress/style.css")
        }
        div {
            style: "display: flex; flex-direction: column; gap: 1rem;",
            Progress {
                value: progress(),
                max: 100.0,
                class: "progress".to_string(),
                aria_label: "Progressbar Demo".to_string()
            }
            span { "Progress: {progress()}%" }
        }
    }
}
