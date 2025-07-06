// Input Components with Unified Theme System
//
// This module provides a comprehensive set of input components that follow modern
// design principles and integrate seamlessly with the unified theme system.
//
// Features:
// - Multiple input variants (filled, outlined, standard)
// - Various input types (text, password, email, number, etc.)
// - Different sizes and states
// - Icon support with flexible positioning
// - Helper text and error messages
// - Full accessibility support
// - Label integration
// - Validation states
// - Theme-aware styling
// - Responsive design

use dioxus::prelude::*;

/// Input variant styles
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputVariant {
    Filled,
    Outlined,
    Standard,
}

impl Default for InputVariant {
    fn default() -> Self {
        Self::Outlined
    }
}

impl InputVariant {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Filled => "input--filled",
            Self::Outlined => "input--outlined",
            Self::Standard => "input--standard",
        }
    }
}

/// Input size variants
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputSize {
    Small,
    Medium,
    Large,
}

impl Default for InputSize {
    fn default() -> Self {
        Self::Medium
    }
}

impl InputSize {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Small => "input--sm",
            Self::Medium => "input--md",
            Self::Large => "input--lg",
        }
    }
}

/// Input validation state
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputState {
    Normal,
    Success,
    Warning,
    Error,
}

impl Default for InputState {
    fn default() -> Self {
        Self::Normal
    }
}

impl InputState {
    pub fn as_class(&self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Success => "input--success",
            Self::Warning => "input--warning",
            Self::Error => "input--error",
        }
    }
}

/// Input type for HTML input element
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
    Search,
    Date,
    Time,
    DatetimeLocal,
    Month,
    Week,
    Color,
}

impl Default for InputType {
    fn default() -> Self {
        Self::Text
    }
}

impl InputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
            Self::Email => "email",
            Self::Number => "number",
            Self::Tel => "tel",
            Self::Url => "url",
            Self::Search => "search",
            Self::Date => "date",
            Self::Time => "time",
            Self::DatetimeLocal => "datetime-local",
            Self::Month => "month",
            Self::Week => "week",
            Self::Color => "color",
        }
    }
}

/// Input component properties
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    /// Input variant
    #[props(optional, default = InputVariant::Outlined)]
    pub variant: InputVariant,

    /// Input size
    #[props(optional, default = InputSize::Medium)]
    pub size: InputSize,

    /// Input type
    #[props(optional, default = InputType::Text)]
    pub input_type: InputType,

    /// Input validation state
    #[props(optional, default = InputState::Normal)]
    pub state: InputState,

    /// Input value
    #[props(optional)]
    pub value: Option<String>,

    /// Placeholder text
    #[props(optional)]
    pub placeholder: Option<String>,

    /// Label text
    #[props(optional)]
    pub label: Option<String>,

    /// Helper text
    #[props(optional)]
    pub helper_text: Option<String>,

    /// Error message
    #[props(optional)]
    pub error_message: Option<String>,

    /// Whether input is required
    #[props(optional, default = false)]
    pub required: bool,

    /// Whether input is disabled
    #[props(optional, default = false)]
    pub disabled: bool,

    /// Whether input is readonly
    #[props(optional, default = false)]
    pub readonly: bool,

    /// Whether input should autofocus
    #[props(optional, default = false)]
    pub autofocus: bool,

    /// Whether input should take full width
    #[props(optional, default = false)]
    pub full_width: bool,

    /// Left icon element
    #[props(optional)]
    pub left_icon: Option<Element>,

    /// Right icon element
    #[props(optional)]
    pub right_icon: Option<Element>,

    /// Minimum value (for number inputs)
    #[props(optional)]
    pub min: Option<String>,

    /// Maximum value (for number inputs)
    #[props(optional)]
    pub max: Option<String>,

    /// Step value (for number inputs)
    #[props(optional)]
    pub step: Option<String>,

    /// Maximum length
    #[props(optional)]
    pub maxlength: Option<u32>,

    /// Pattern for validation
    #[props(optional)]
    pub pattern: Option<String>,

    /// Autocomplete attribute
    #[props(optional)]
    pub autocomplete: Option<String>,

    /// Input change event handler
    #[props(optional)]
    pub oninput: Option<EventHandler<FormEvent>>,

    /// Input change event handler
    #[props(optional)]
    pub onchange: Option<EventHandler<FormEvent>>,

    /// Focus event handler
    #[props(optional)]
    pub onfocus: Option<EventHandler<FocusEvent>>,

    /// Blur event handler
    #[props(optional)]
    pub onblur: Option<EventHandler<FocusEvent>>,

    /// Key down event handler
    #[props(optional)]
    pub onkeydown: Option<EventHandler<KeyboardEvent>>,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,

    /// Input ID
    #[props(optional)]
    pub id: Option<String>,

    /// Input name
    #[props(optional)]
    pub name: Option<String>,

    /// Form ID this input belongs to
    #[props(optional)]
    pub form: Option<String>,

    /// Accessibility label
    #[props(optional)]
    pub aria_label: Option<String>,

    /// Accessibility description
    #[props(optional)]
    pub aria_describedby: Option<String>,

    /// Tab index for keyboard navigation
    #[props(optional)]
    pub tabindex: Option<i32>,

    /// Custom data attributes
    #[props(optional)]
    pub data_testid: Option<String>,
}

/// Enhanced Input Component
#[component]
pub fn Input(props: InputProps) -> Element {
    let InputProps {
        variant,
        size,
        input_type,
        state,
        value,
        placeholder,
        label,
        helper_text,
        error_message,
        required,
        disabled,
        readonly,
        autofocus,
        full_width,
        left_icon,
        right_icon,
        min,
        max,
        step,
        maxlength,
        pattern,
        autocomplete,
        oninput,
        onchange,
        onfocus,
        onblur,
        onkeydown,
        class,
        id,
        name,
        form,
        aria_label,
        aria_describedby,
        tabindex,
        data_testid,
    } = props;

    // Generate unique ID if not provided
    let input_id = id
        .clone()
        .unwrap_or_else(|| format!("input-{}", rand::random::<u32>()));
    let helper_id = format!("{}-helper", input_id);
    let error_id = format!("{}-error", input_id);

    // Build CSS classes
    let mut classes = vec!["input-wrapper"];

    if full_width {
        classes.push("input-wrapper--full-width");
    }

    if disabled {
        classes.push("input-wrapper--disabled");
    }

    if readonly {
        classes.push("input-wrapper--readonly");
    }

    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }

    let wrapper_class = classes.join(" ");

    // Build input classes
    let mut input_classes = vec!["input", variant.as_class(), size.as_class()];

    if !state.as_class().is_empty() {
        input_classes.push(state.as_class());
    }

    if left_icon.is_some() {
        input_classes.push("input--has-left-icon");
    }

    if right_icon.is_some() {
        input_classes.push("input--has-right-icon");
    }

    let input_class = input_classes.join(" ");

    // Determine ARIA describedby
    let aria_described_by = {
        let mut described_by = Vec::new();

        if helper_text.is_some() {
            described_by.push(helper_id.clone());
        }

        if error_message.is_some() {
            described_by.push(error_id.clone());
        }

        if let Some(custom_describedby) = aria_describedby {
            described_by.push(custom_describedby);
        }

        if described_by.is_empty() {
            None
        } else {
            Some(described_by.join(" "))
        }
    };

    // Show error message if present, otherwise show helper text
    let display_message = error_message.as_ref().or(helper_text.as_ref());
    let message_id = if error_message.is_some() {
        &error_id
    } else {
        &helper_id
    };
    let message_class = if error_message.is_some() {
        "input__message input__message--error"
    } else {
        "input__message input__message--helper"
    };

    rsx! {
        // Include CSS styles
        document::Link {
            rel: "stylesheet",
            href: asset!("src/ui/components/input/style.css")
        }

        div { class: wrapper_class,
            // Label
            if let Some(label_text) = label {
                label {
                    class: "input__label",
                    r#for: input_id.clone(),
                    "{label_text}"
                    if required {
                        span { class: "input__required", "*" }
                    }
                }
            }

            // Input container
            div { class: "input__container",
                // Left icon
                if let Some(left_icon_elem) = left_icon {
                    div { class: "input__icon input__icon--left",
                        {left_icon_elem}
                    }
                }

                // Input element
                input {
                    class: input_class,
                    r#type: input_type.as_str(),
                    id: input_id,
                    name: name.unwrap_or_default(),
                    value: value.as_deref().unwrap_or(""),
                    placeholder: placeholder.as_deref().unwrap_or(""),
                    disabled: disabled,
                    readonly: readonly,
                    required: required,
                    autofocus: autofocus,
                    min: min.unwrap_or_default(),
                    max: max.unwrap_or_default(),
                    step: step.unwrap_or_default(),
                    maxlength: maxlength.map(|l| l.to_string()).unwrap_or_default(),
                    pattern: pattern.unwrap_or_default(),
                    autocomplete: autocomplete.unwrap_or_default(),
                    form: form.unwrap_or_default(),
                    tabindex: tabindex.unwrap_or(0),
                    "data-testid": data_testid.unwrap_or_default(),

                    // Accessibility
                    aria_label: aria_label.unwrap_or_default(),
                    aria_describedby: aria_described_by.unwrap_or_default(),
                    aria_invalid: if matches!(state, InputState::Error) { "true" } else { "false" },
                    aria_required: required.to_string(),

                    // Event handlers
                    oninput: move |evt| {
                        if let Some(handler) = &oninput {
                            handler.call(evt);
                        }
                    },
                    onchange: move |evt| {
                        if let Some(handler) = &onchange {
                            handler.call(evt);
                        }
                    },
                    onfocus: move |evt| {
                        if let Some(handler) = &onfocus {
                            handler.call(evt);
                        }
                    },
                    onblur: move |evt| {
                        if let Some(handler) = &onblur {
                            handler.call(evt);
                        }
                    },
                    onkeydown: move |evt| {
                        if let Some(handler) = &onkeydown {
                            handler.call(evt);
                        }
                    }
                }

                // Right icon
                if let Some(right_icon_elem) = right_icon {
                    div { class: "input__icon input__icon--right",
                        {right_icon_elem}
                    }
                }
            }

            // Helper text or error message
            if let Some(message) = display_message {
                div {
                    class: message_class,
                    id: message_id.as_str(),
                    "{message}"
                }
            }
        }
    }
}

/// TextArea component properties
#[derive(Props, Clone, PartialEq)]
pub struct TextAreaProps {
    /// TextArea variant
    #[props(optional, default = InputVariant::Outlined)]
    pub variant: InputVariant,

    /// TextArea size
    #[props(optional, default = InputSize::Medium)]
    pub size: InputSize,

    /// TextArea validation state
    #[props(optional, default = InputState::Normal)]
    pub state: InputState,

    /// TextArea value
    #[props(optional)]
    pub value: Option<String>,

    /// Placeholder text
    #[props(optional)]
    pub placeholder: Option<String>,

    /// Label text
    #[props(optional)]
    pub label: Option<String>,

    /// Helper text
    #[props(optional)]
    pub helper_text: Option<String>,

    /// Error message
    #[props(optional)]
    pub error_message: Option<String>,

    /// Number of rows
    #[props(optional, default = 4)]
    pub rows: u32,

    /// Number of columns
    #[props(optional)]
    pub cols: Option<u32>,

    /// Whether textarea is required
    #[props(optional, default = false)]
    pub required: bool,

    /// Whether textarea is disabled
    #[props(optional, default = false)]
    pub disabled: bool,

    /// Whether textarea is readonly
    #[props(optional, default = false)]
    pub readonly: bool,

    /// Whether textarea should autofocus
    #[props(optional, default = false)]
    pub autofocus: bool,

    /// Whether textarea should take full width
    #[props(optional, default = false)]
    pub full_width: bool,

    /// Maximum length
    #[props(optional)]
    pub maxlength: Option<u32>,

    /// Whether textarea should resize
    #[props(optional, default = true)]
    pub resizable: bool,

    /// TextArea input event handler
    #[props(optional)]
    pub oninput: Option<EventHandler<FormEvent>>,

    /// TextArea change event handler
    #[props(optional)]
    pub onchange: Option<EventHandler<FormEvent>>,

    /// Additional CSS classes
    #[props(optional)]
    pub class: Option<String>,

    /// TextArea ID
    #[props(optional)]
    pub id: Option<String>,

    /// TextArea name
    #[props(optional)]
    pub name: Option<String>,
}

/// TextArea Component
#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
    let TextAreaProps {
        variant,
        size,
        state,
        value,
        placeholder,
        label,
        helper_text,
        error_message,
        rows,
        cols,
        required,
        disabled,
        readonly,
        autofocus,
        full_width,
        maxlength,
        resizable,
        oninput,
        onchange,
        class,
        id,
        name,
    } = props;

    // Generate unique ID if not provided
    let textarea_id = id
        .clone()
        .unwrap_or_else(|| format!("textarea-{}", rand::random::<u32>()));
    let helper_id = format!("{}-helper", textarea_id);
    let error_id = format!("{}-error", textarea_id);

    // Build CSS classes
    let mut classes = vec!["input-wrapper"];

    if full_width {
        classes.push("input-wrapper--full-width");
    }

    if disabled {
        classes.push("input-wrapper--disabled");
    }

    if let Some(custom_class) = &class {
        classes.push(custom_class);
    }

    let wrapper_class = classes.join(" ");

    // Build textarea classes
    let mut textarea_classes = vec!["textarea", variant.as_class(), size.as_class()];

    if !state.as_class().is_empty() {
        textarea_classes.push(state.as_class());
    }

    if !resizable {
        textarea_classes.push("textarea--no-resize");
    }

    let textarea_class = textarea_classes.join(" ");

    // Show error message if present, otherwise show helper text
    let display_message = error_message.as_ref().or(helper_text.as_ref());
    let message_id = if error_message.is_some() {
        &error_id
    } else {
        &helper_id
    };
    let message_class = if error_message.is_some() {
        "input__message input__message--error"
    } else {
        "input__message input__message--helper"
    };

    rsx! {
        div { class: wrapper_class,
            // Label
            if let Some(label_text) = label {
                label {
                    class: "input__label",
                    r#for: textarea_id.clone(),
                    "{label_text}"
                    if required {
                        span { class: "input__required", "*" }
                    }
                }
            }

            // TextArea container
            div { class: "input__container",
                // TextArea element
                textarea {
                    class: textarea_class,
                    id: textarea_id,
                    name: name.unwrap_or_default(),
                    value: value.as_deref().unwrap_or(""),
                    placeholder: placeholder.as_deref().unwrap_or(""),
                    rows: rows.to_string(),
                    cols: cols.map(|c| c.to_string()).unwrap_or_default(),
                    disabled: disabled,
                    readonly: readonly,
                    required: required,
                    autofocus: autofocus,
                    maxlength: maxlength.map(|l| l.to_string()).unwrap_or_default(),

                    // Accessibility
                    aria_describedby: if display_message.is_some() { message_id.as_str() } else { "" },
                    aria_invalid: if matches!(state, InputState::Error) { "true" } else { "false" },
                    aria_required: required.to_string(),

                    // Event handlers
                    oninput: move |evt| {
                        if let Some(handler) = &oninput {
                            handler.call(evt);
                        }
                    },
                    onchange: move |evt| {
                        if let Some(handler) = &onchange {
                            handler.call(evt);
                        }
                    }
                }
            }

            // Helper text or error message
            if let Some(message) = display_message {
                div {
                    class: message_class,
                    id: message_id.as_str(),
                    "{message}"
                }
            }
        }
    }
}

/// Search Input - specialized input for search functionality
#[component]
pub fn SearchInput(
    #[props(optional)] value: Option<String>,
    #[props(optional)] placeholder: Option<String>,
    #[props(optional)] oninput: Option<EventHandler<FormEvent>>,
    #[props(optional)] onchange: Option<EventHandler<FormEvent>>,
    #[props(optional)] variant: Option<InputVariant>,
    #[props(optional)] size: Option<InputSize>,
    #[props(optional)] full_width: Option<bool>,
    #[props(optional)] class: Option<String>,
) -> Element {
    rsx! {
        Input {
            input_type: InputType::Search,
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            value,
            placeholder: placeholder.or(Some("Search...".to_string())),
            full_width: full_width.unwrap_or(false),
            left_icon: rsx! { span { "🔍" } },
            oninput,
            onchange,
            class,
            autocomplete: Some("off".to_string())
        }
    }
}

/// Password Input - specialized input for passwords with show/hide toggle
#[component]
pub fn PasswordInput(
    #[props(optional)] value: Option<String>,
    #[props(optional)] placeholder: Option<String>,
    #[props(optional)] oninput: Option<EventHandler<FormEvent>>,
    #[props(optional)] onchange: Option<EventHandler<FormEvent>>,
    #[props(optional)] variant: Option<InputVariant>,
    #[props(optional)] size: Option<InputSize>,
    #[props(optional)] full_width: Option<bool>,
    #[props(optional)] required: Option<bool>,
    #[props(optional)] class: Option<String>,
    #[props(optional)] label: Option<String>,
) -> Element {
    let mut show_password = use_signal(|| false);

    let toggle_password = move |_| {
        show_password.set(!show_password());
    };

    let input_type = if *show_password.read() {
        InputType::Text
    } else {
        InputType::Password
    };

    let toggle_icon = if *show_password.read() {
        rsx! {
            button {
                r#type: "button",
                onclick: toggle_password,
                aria_label: "Hide password",
                style: "background: none; border: none; cursor: pointer; padding: 0.25rem; display: flex; align-items: center;",
                "👁️"
            }
        }
    } else {
        rsx! {
            button {
                r#type: "button",
                onclick: toggle_password,
                aria_label: "Show password",
                style: "background: none; border: none; cursor: pointer; padding: 0.25rem; display: flex; align-items: center;",
                "🙈"
            }
        }
    };

    rsx! {
        Input {
            input_type,
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            value,
            placeholder: placeholder.or(Some("Enter password...".to_string())),
            full_width: full_width.unwrap_or(false),
            required: required.unwrap_or(false),
            right_icon: toggle_icon,
            oninput,
            onchange,
            class,
            label,
            autocomplete: Some("current-password".to_string())
        }
    }
}

/// Number Input - specialized input for numeric values
#[component]
pub fn NumberInput(
    #[props(optional)] value: Option<String>,
    #[props(optional)] min: Option<String>,
    #[props(optional)] max: Option<String>,
    #[props(optional)] step: Option<String>,
    #[props(optional)] placeholder: Option<String>,
    #[props(optional)] oninput: Option<EventHandler<FormEvent>>,
    #[props(optional)] onchange: Option<EventHandler<FormEvent>>,
    #[props(optional)] variant: Option<InputVariant>,
    #[props(optional)] size: Option<InputSize>,
    #[props(optional)] full_width: Option<bool>,
    #[props(optional)] required: Option<bool>,
    #[props(optional)] disabled: Option<bool>,
    #[props(optional)] class: Option<String>,
    #[props(optional)] label: Option<String>,
) -> Element {
    rsx! {
        Input {
            input_type: InputType::Number,
            variant: variant.unwrap_or_default(),
            size: size.unwrap_or_default(),
            value,
            min,
            max,
            step: step.or(Some("1".to_string())),
            placeholder,
            full_width: full_width.unwrap_or(false),
            required: required.unwrap_or(false),
            disabled: disabled.unwrap_or(false),
            oninput,
            onchange,
            class,
            label
        }
    }
}

/// Demo component showcasing all input variants and states
#[component]
pub(crate) fn Demo() -> Element {
    let mut text_value = use_signal(|| "".to_string());
    let mut email_value = use_signal(|| "".to_string());
    let mut password_value = use_signal(|| "".to_string());
    let mut search_value = use_signal(|| "".to_string());
    let mut number_value = use_signal(|| "".to_string());
    let mut textarea_value = use_signal(|| "".to_string());

    rsx! {
        div {
            style: "padding: 2rem; max-width: 1200px; margin: 0 auto;",

            h2 { "Input Component Demo" }

            // Basic input variants
            section {
                h3 { "Input Variants" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    Input {
                        variant: InputVariant::Filled,
                        label: Some("Filled Input".to_string()),
                        placeholder: Some("Enter text...".to_string()),
                        value: Some(text_value().clone()),
                        oninput: move |evt: FormEvent| text_value.set(evt.value()),
                    }

                    Input {
                        variant: InputVariant::Outlined,
                        label: Some("Outlined Input".to_string()),
                        placeholder: Some("Enter text...".to_string()),
                        helper_text: Some("This is helper text".to_string())
                    }

                    Input {
                        variant: InputVariant::Standard,
                        label: Some("Standard Input".to_string()),
                        placeholder: Some("Enter text...".to_string())
                    }
                }
            }

            // Input sizes
            section {
                h3 { "Input Sizes" }
                div { style: "display: flex; flex-direction: column; gap: 1rem; margin-bottom: 2rem;",

                    Input {
                        size: InputSize::Small,
                        label: Some("Small Input".to_string()),
                        placeholder: Some("Small size".to_string())
                    }

                    Input {
                        size: InputSize::Medium,
                        label: Some("Medium Input".to_string()),
                        placeholder: Some("Medium size".to_string())
                    }

                    Input {
                        size: InputSize::Large,
                        label: Some("Large Input".to_string()),
                        placeholder: Some("Large size".to_string())
                    }
                }
            }

            // Input states
            section {
                h3 { "Input States" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    Input {
                        state: InputState::Success,
                        label: Some("Success State".to_string()),
                        value: Some("Valid input".to_string()),
                        helper_text: Some("Input is valid".to_string())
                    }

                    Input {
                        state: InputState::Warning,
                        label: Some("Warning State".to_string()),
                        value: Some("Warning input".to_string()),
                        helper_text: Some("Please check this input".to_string())
                    }

                    Input {
                        state: InputState::Error,
                        label: Some("Error State".to_string()),
                        value: Some("Invalid input".to_string()),
                        error_message: Some("This field is required".to_string()),
                        required: true
                    }

                    Input {
                        label: Some("Disabled Input".to_string()),
                        value: Some("Disabled value".to_string()),
                        disabled: true
                    }
                }
            }

            // Input types and specialized inputs
            section {
                h3 { "Input Types" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    Input {
                        input_type: InputType::Email,
                        label: Some("Email Input".to_string()),
                        placeholder: Some("user@example.com".to_string()),
                        value: Some(email_value().clone()),
                        oninput: move |evt: FormEvent| email_value.set(evt.value()),
                        left_icon: rsx! { span { "✉️" } }
                    }

                    PasswordInput {
                        label: Some("Password Input".to_string()),
                        value: Some(password_value().clone()),
                        oninput: move |evt: FormEvent| password_value.set(evt.value()),
                        required: Some(true)
                    }

                    SearchInput {
                        value: Some(search_value().clone()),
                        oninput: move |evt: FormEvent| search_value.set(evt.value()),
                        full_width: Some(true)
                    }

                    NumberInput {
                        label: Some("Number Input".to_string()),
                        value: Some(number_value().clone()),
                        min: Some("0".to_string()),
                        max: Some("100".to_string()),
                        step: Some("1".to_string()),
                        oninput: move |evt: FormEvent| number_value.set(evt.value()),
                    }
                }
            }

            // TextArea examples
            section {
                h3 { "TextArea Component" }
                div { style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1rem; margin-bottom: 2rem;",

                    TextArea {
                        label: Some("Message".to_string()),
                        placeholder: Some("Enter your message...".to_string()),
                        value: Some(textarea_value().clone()),
                        oninput: move |evt: FormEvent| textarea_value.set(evt.value()),
                        rows: 4u32,
                        helper_text: Some("Maximum 500 characters".to_string()),
                        maxlength: Some(500)
                    }

                    TextArea {
                        variant: InputVariant::Filled,
                        label: Some("Filled TextArea".to_string()),
                        placeholder: Some("Filled variant...".to_string()),
                        rows: 6u32,
                        resizable: false
                    }

                    TextArea {
                        variant: InputVariant::Standard,
                        label: Some("Standard TextArea".to_string()),
                        placeholder: Some("Standard variant...".to_string()),
                        rows: 5u32,
                        required: true
                    }
                }
            }

            // Form example
            section {
                h3 { "Complete Form Example" }
                div { style: "max-width: 500px; margin-bottom: 2rem;",
                    form {
                        style: "display: flex; flex-direction: column; gap: 1rem;",

                        Input {
                            input_type: InputType::Text,
                            label: Some("Full Name".to_string()),
                            placeholder: Some("Enter your full name".to_string()),
                            required: true,
                            full_width: true
                        }

                        Input {
                            input_type: InputType::Email,
                            label: Some("Email Address".to_string()),
                            placeholder: Some("your.email@example.com".to_string()),
                            required: true,
                            full_width: true,
                            left_icon: rsx! { span { "✉️" } }
                        }

                        PasswordInput {
                            label: Some("Password".to_string()),
                            required: Some(true),
                            full_width: Some(true)
                        }

                        NumberInput {
                            label: Some("Age".to_string()),
                            min: Some("18".to_string()),
                            max: Some("120".to_string()),
                            required: Some(true),
                            full_width: Some(true)
                        }

                        TextArea {
                            label: Some("Bio".to_string()),
                            placeholder: Some("Tell us about yourself...".to_string()),
                            rows: 4u32,
                            full_width: true,
                            helper_text: Some("Optional - share a bit about yourself".to_string())
                        }

                        div { style: "display: flex; gap: 1rem; justify-content: flex-end; margin-top: 1rem;",
                            button {
                                r#type: "button",
                                style: "padding: 0.75rem 1.5rem; border: 1px solid var(--border-primary); background: transparent; border-radius: var(--radius-md); cursor: pointer;",
                                "Cancel"
                            }
                            button {
                                r#type: "submit",
                                style: "padding: 0.75rem 1.5rem; background: var(--color-primary); color: white; border: none; border-radius: var(--radius-md); cursor: pointer;",
                                "Submit"
                            }
                        }
                    }
                }
            }

            // Current values display
            section {
                h3 { "Current Values" }
                div { style: "background: var(--bg-secondary); padding: 1rem; border-radius: var(--radius-md); font-family: monospace;",
                    p { "Text: '{text_value()}'" }
                    p { "Email: '{email_value()}'" }
                    p { "Password: '{password_value()}'" }
                    p { "Search: '{search_value()}'" }
                    p { "Number: '{number_value()}'" }
                    p { "TextArea: '{textarea_value()}'" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_variant_classes() {
        assert_eq!(InputVariant::Filled.as_class(), "input--filled");
        assert_eq!(InputVariant::Outlined.as_class(), "input--outlined");
        assert_eq!(InputVariant::Standard.as_class(), "input--standard");
    }

    #[test]
    fn test_input_size_classes() {
        assert_eq!(InputSize::Small.as_class(), "input--sm");
        assert_eq!(InputSize::Medium.as_class(), "input--md");
        assert_eq!(InputSize::Large.as_class(), "input--lg");
    }

    #[test]
    fn test_input_state_classes() {
        assert_eq!(InputState::Normal.as_class(), "");
        assert_eq!(InputState::Success.as_class(), "input--success");
        assert_eq!(InputState::Warning.as_class(), "input--warning");
        assert_eq!(InputState::Error.as_class(), "input--error");
    }

    #[test]
    fn test_input_type_strings() {
        assert_eq!(InputType::Text.as_str(), "text");
        assert_eq!(InputType::Password.as_str(), "password");
        assert_eq!(InputType::Email.as_str(), "email");
        assert_eq!(InputType::Number.as_str(), "number");
        assert_eq!(InputType::Search.as_str(), "search");
    }

    #[test]
    fn test_defaults() {
        assert_eq!(InputVariant::default(), InputVariant::Outlined);
        assert_eq!(InputSize::default(), InputSize::Medium);
        assert_eq!(InputState::default(), InputState::Normal);
        assert_eq!(InputType::default(), InputType::Text);
    }
}
