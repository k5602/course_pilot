pub fn load_theme() {
    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/com/coursepilot/app/css/style.css");

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    log::info!("GTK theme loaded from GResource");
}
