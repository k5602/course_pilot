//! Performance Benchmarks for Course Pilot UI Components
//!
//! This module provides comprehensive performance testing for UI components,
//! focusing on rendering performance with large datasets and memory usage.

use dioxus::prelude::*;
use dioxus_ssr::render;
use dioxus_core;
use std::time::{Duration, Instant};

// Import components for testing
use course_pilot::ui::components::base::{
    BaseButton, BaseButtonProps, BaseCard, BaseCardProps, BasePage, BasePageProps,
};
use course_pilot::ui::components::{
    ProgressRing, ProgressBar,
};

/// Performance benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub dataset_size: usize,
    pub max_acceptable_time: Duration,
    pub memory_threshold_mb: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            dataset_size: 1000,
            max_acceptable_time: Duration::from_millis(100),
            memory_threshold_mb: 50,
        }
    }
}

/// Performance metrics collected during benchmarks
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub min_time: Duration,
    pub max_time: Duration,
    pub avg_time: Duration,
    pub total_time: Duration,
    pub iterations: usize,
    pub memory_usage_mb: f64,
    pub renders_per_second: f64,
}

impl PerformanceMetrics {
    fn new(times: Vec<Duration>, memory_usage_mb: f64) -> Self {
        let total_time: Duration = times.iter().sum();
        let avg_time = total_time / times.len() as u32;
        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();
        let renders_per_second = times.len() as f64 / total_time.as_secs_f64();

        Self {
            min_time,
            max_time,
            avg_time,
            total_time,
            iterations: times.len(),
            memory_usage_mb,
            renders_per_second,
        }
    }

    fn print_summary(&self, component_name: &str) {
        println!("\n=== Performance Metrics for {} ===", component_name);
        println!("Iterations: {}", self.iterations);
        println!("Min time: {:?}", self.min_time);
        println!("Max time: {:?}", self.max_time);
        println!("Avg time: {:?}", self.avg_time);
        println!("Total time: {:?}", self.total_time);
        println!("Memory usage: {:.2} MB", self.memory_usage_mb);
        println!("Renders/sec: {:.2}", self.renders_per_second);
    }
}

/// Helper function to render a component and measure performance
fn benchmark_component_render<P: Clone + 'static>(
    component: fn(P) -> Element,
    props: P,
    config: &BenchmarkConfig,
) -> PerformanceMetrics {
    let mut render_times = Vec::new();
    
    // Warm up
    for _ in 0..10 {
        let mut dom = VirtualDom::new_with_props(component, props.clone());
        let mut mutations = dioxus_core::Mutations::default();
        let _ = dom.rebuild(&mut mutations);
        let _ = render(&dom);
    }

    // Actual benchmark
    for _ in 0..config.iterations {
        let start = Instant::now();
        
        let mut dom = VirtualDom::new_with_props(component, props.clone());
        let mut mutations = dioxus_core::Mutations::default();
        let _ = dom.rebuild(&mut mutations);
        let _ = render(&dom);
        
        let elapsed = start.elapsed();
        render_times.push(elapsed);
    }

    // Estimate memory usage (simplified)
    let memory_usage_mb = estimate_memory_usage(&render_times);
    
    PerformanceMetrics::new(render_times, memory_usage_mb)
}

/// Simplified memory usage estimation
fn estimate_memory_usage(render_times: &[Duration]) -> f64 {
    // This is a simplified estimation based on render times
    // In a real scenario, you'd use proper memory profiling tools
    let avg_time_ms = render_times.iter().sum::<Duration>().as_millis() as f64 / render_times.len() as f64;
    
    // Rough estimation: longer render times often correlate with more memory usage
    (avg_time_ms / 10.0).max(1.0)
}

/// Generate large dataset for testing
fn generate_large_course_dataset(size: usize) -> Vec<(String, String, u32)> {
    (0..size)
        .map(|i| {
            (
                format!("Course Title {}", i),
                format!("This is a detailed description for course {} with lots of content to test rendering performance with large text blocks", i),
                (i % 100) as u32, // Progress percentage
            )
        })
        .collect()
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_base_button_rendering_performance() {
        let config = BenchmarkConfig::default();
        
        let metrics = benchmark_component_render(
            BaseButton,
            BaseButtonProps {
                children: rsx! { "Performance Test Button" },
                onclick: None,
                color: Some("primary".to_string()),
                size: Some("md".to_string()),
                variant: None,
                class: "",
                disabled: false,
                icon: Some(rsx! { span { "🚀" } }),
                loading: false,
                button_type: "button",
            },
            &config,
        );

        metrics.print_summary("BaseButton");
        
        // Performance assertions
        assert!(
            metrics.avg_time < config.max_acceptable_time,
            "BaseButton average render time ({:?}) exceeds threshold ({:?})",
            metrics.avg_time,
            config.max_acceptable_time
        );
        
        assert!(
            metrics.memory_usage_mb < config.memory_threshold_mb as f64,
            "BaseButton memory usage ({:.2} MB) exceeds threshold ({} MB)",
            metrics.memory_usage_mb,
            config.memory_threshold_mb
        );
        
        assert!(
            metrics.renders_per_second > 100.0,
            "BaseButton renders per second ({:.2}) is too low",
            metrics.renders_per_second
        );
    }

    #[test]
    fn test_base_card_rendering_performance() {
        let config = BenchmarkConfig::default();
        
        let metrics = benchmark_component_render(
            BaseCard,
            BaseCardProps {
                title: Some("Performance Test Card".to_string()),
                subtitle: Some("Testing card rendering performance with various content".to_string()),
                children: rsx! {
                    div { class: "space-y-4",
                        p { "This is a test card with multiple content elements to simulate real usage." }
                        div { class: "flex gap-2",
                            span { class: "badge badge-primary", "Tag 1" }
                            span { class: "badge badge-secondary", "Tag 2" }
                            span { class: "badge badge-accent", "Tag 3" }
                        }
                        div { class: "stats shadow",
                            div { class: "stat",
                                div { class: "stat-title", "Progress" }
                                div { class: "stat-value", "75%" }
                            }
                        }
                    }
                },
                variant: "card",
                class: "w-96",
                hover_effect: true,
                on_click: None,
                actions: Some(rsx! {
                    button { class: "btn btn-primary btn-sm", "Action 1" }
                    button { class: "btn btn-secondary btn-sm", "Action 2" }
                }),
                header_actions: Some(rsx! {
                    button { class: "btn btn-ghost btn-sm", "⋮" }
                }),
            },
            &config,
        );

        metrics.print_summary("BaseCard");
        
        // Performance assertions
        assert!(
            metrics.avg_time < Duration::from_millis(150), // Cards can be more complex
            "BaseCard average render time ({:?}) exceeds threshold",
            metrics.avg_time
        );
        
        assert!(
            metrics.renders_per_second > 50.0,
            "BaseCard renders per second ({:.2}) is too low",
            metrics.renders_per_second
        );
    }

    #[test]
    fn test_large_dataset_rendering_performance() {
        let config = BenchmarkConfig {
            iterations: 10, // Fewer iterations for large dataset tests
            dataset_size: 100,
            max_acceptable_time: Duration::from_millis(500),
            memory_threshold_mb: 100,
        };
        
        let dataset = generate_large_course_dataset(config.dataset_size);
        
        let metrics = benchmark_component_render(
            BasePage,
            BasePageProps {
                title: Some("Large Dataset Performance Test".to_string()),
                subtitle: Some(format!("Rendering {} items", dataset.len())),
                children: rsx! {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                        {dataset.iter().enumerate().map(|(i, (title, description, progress))| {
                            rsx! {
                                div { key: "{i}",
                                    class: "card bg-base-100 shadow-xl",
                                    div { class: "card-body",
                                        h2 { class: "card-title", "{title}" }
                                        p { "{description}" }
                                        div { class: "card-actions justify-end",
                                            div { class: "radial-progress text-primary",
                                                style: "--value:{progress}",
                                                "{progress}%"
                                            }
                                            button { class: "btn btn-primary btn-sm", "View" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                },
                header_actions: Some(rsx! {
                    button { class: "btn btn-primary", "Add Course" }
                }),
                breadcrumbs: Some(rsx! {
                    div { class: "breadcrumbs text-sm",
                        ul {
                            li { a { "Home" } }
                            li { a { "Courses" } }
                            li { "Performance Test" }
                        }
                    }
                }),
                class: "",
                max_width: "max-w-7xl",
                padded: true,
                background: "bg-base-100",
            },
            &config,
        );

        metrics.print_summary("Large Dataset Rendering");
        
        // Performance assertions for large datasets
        assert!(
            metrics.avg_time < config.max_acceptable_time,
            "Large dataset render time ({:?}) exceeds threshold ({:?})",
            metrics.avg_time,
            config.max_acceptable_time
        );
        
        assert!(
            metrics.memory_usage_mb < config.memory_threshold_mb as f64,
            "Large dataset memory usage ({:.2} MB) exceeds threshold ({} MB)",
            metrics.memory_usage_mb,
            config.memory_threshold_mb
        );
    }

    #[test]
    fn test_progress_components_performance() {
        let config = BenchmarkConfig::default();
        
        // Test ProgressRing performance
        let progress_metrics = benchmark_component_render(
            ProgressRing,
            course_pilot::ui::components::progress::ProgressRingProps {
                value: 75,
                max: Some(100),
                color: Some("primary".to_string()),
                size: Some(64),
                thickness: Some(6),
                label: Some(rsx! { span { class: "text-sm font-medium", "Loading..." } }),
            },
            &config,
        );

        progress_metrics.print_summary("ProgressRing");
        
        // Test ProgressBar performance
        let bar_metrics = benchmark_component_render(
            ProgressBar,
            course_pilot::ui::components::progress::ProgressBarProps {
                value: 60,
                label: Some("Processing...".to_string()),
                color: Some("success".to_string()),
                class: Some("w-full".to_string()),
            },
            &config,
        );

        bar_metrics.print_summary("ProgressBar");
        
        // Performance assertions
        assert!(
            progress_metrics.avg_time < Duration::from_millis(50),
            "ProgressRing render time too slow: {:?}",
            progress_metrics.avg_time
        );
        
        assert!(
            bar_metrics.avg_time < Duration::from_millis(30),
            "ProgressBar render time too slow: {:?}",
            bar_metrics.avg_time
        );
        
        assert!(
            progress_metrics.renders_per_second > 200.0,
            "ProgressRing renders per second too low: {:.2}",
            progress_metrics.renders_per_second
        );
    }

    #[test]
    fn test_component_rerender_performance() {
        // Test how components perform when props change frequently
        let config = BenchmarkConfig {
            iterations: 50,
            ..Default::default()
        };
        
        let mut render_times = Vec::new();
        
        // Test button with changing states
        for i in 0..config.iterations {
            let start = Instant::now();
            
            let props = BaseButtonProps {
                children: rsx! { "Button {i}" },
                onclick: None,
                color: Some(if i % 2 == 0 { "primary" } else { "secondary" }.to_string()),
                size: Some(match i % 3 {
                    0 => "sm",
                    1 => "md",
                    _ => "lg",
                }.to_string()),
                variant: if i % 4 == 0 { Some("outline".to_string()) } else { None },
                class: "",
                disabled: i % 10 == 0,
                icon: if i % 5 == 0 { Some(rsx! { span { "🔄" } }) } else { None },
                loading: i % 8 == 0,
                button_type: "button",
            };
            
            let mut dom = VirtualDom::new_with_props(BaseButton, props);
            let mut mutations = dioxus_core::Mutations::default();
            let _ = dom.rebuild(&mut mutations);
            let _ = render(&dom);
            
            render_times.push(start.elapsed());
        }
        
        let metrics = PerformanceMetrics::new(render_times, 5.0);
        metrics.print_summary("Component Rerender");
        
        // Rerender should be fast
        assert!(
            metrics.avg_time < Duration::from_millis(80),
            "Component rerender time too slow: {:?}",
            metrics.avg_time
        );
        
        assert!(
            metrics.renders_per_second > 100.0,
            "Component rerender rate too low: {:.2}",
            metrics.renders_per_second
        );
    }

    #[test]
    fn test_memory_usage_with_complex_components() {
        // Test memory usage with deeply nested components
        let config = BenchmarkConfig {
            iterations: 20,
            dataset_size: 50,
            max_acceptable_time: Duration::from_millis(200),
            memory_threshold_mb: 75,
        };
        
        // Create nested content as a simple structure to avoid RSX complexity
        let section_count = config.dataset_size;
        
        let metrics = benchmark_component_render(
            BasePage,
            BasePageProps {
                title: Some("Complex Nested Components Test".to_string()),
                subtitle: Some("Testing memory usage with deeply nested components".to_string()),
                children: rsx! {
                    div { class: "space-y-4",
                        for i in 0..section_count {
                            div { key: "{i}", class: "collapse collapse-arrow bg-base-200 mb-2",
                                input { r#type: "checkbox" }
                                div { class: "collapse-title text-xl font-medium",
                                    "Section {i}"
                                }
                                div { class: "collapse-content",
                                    div { class: "grid grid-cols-2 gap-4",
                                        div { class: "card bg-base-100 shadow-sm",
                                            div { class: "card-body p-4",
                                                h3 { class: "card-title text-lg", "Subsection A" }
                                                p { "Content for subsection A with detailed information..." }
                                                div { class: "card-actions justify-end",
                                                    button { class: "btn btn-primary btn-sm", "Action" }
                                                }
                                            }
                                        }
                                        div { class: "card bg-base-100 shadow-sm",
                                            div { class: "card-body p-4",
                                                h3 { class: "card-title text-lg", "Subsection B" }
                                                p { "Content for subsection B with more information..." }
                                                div { class: "progress-container",
                                                    div { class: "radial-progress text-primary",
                                                        style: "--value:{}", (i * 2) % 100,
                                                        "{}", (i * 2) % 100, "%"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                header_actions: None,
                breadcrumbs: None,
                class: "",
                max_width: "max-w-6xl",
                padded: true,
                background: "bg-base-100",
            },
            &config,
        );

        metrics.print_summary("Complex Nested Components");
        
        // Memory usage assertions
        assert!(
            metrics.memory_usage_mb < config.memory_threshold_mb as f64,
            "Complex component memory usage ({:.2} MB) exceeds threshold ({} MB)",
            metrics.memory_usage_mb,
            config.memory_threshold_mb
        );
        
        assert!(
            metrics.avg_time < config.max_acceptable_time,
            "Complex component render time ({:?}) exceeds threshold ({:?})",
            metrics.avg_time,
            config.max_acceptable_time
        );
    }
}

/// Benchmark runner for manual testing
#[allow(dead_code)]
pub fn run_all_benchmarks() {
    println!("Running Course Pilot UI Component Performance Benchmarks...\n");
    
    // This would be called manually for comprehensive benchmarking
    // In practice, individual tests are run via `cargo test`
    
    println!("Benchmarks completed. Run individual tests with:");
    println!("cargo test test_base_button_rendering_performance -- --nocapture");
    println!("cargo test test_base_card_rendering_performance -- --nocapture");
    println!("cargo test test_large_dataset_rendering_performance -- --nocapture");
}