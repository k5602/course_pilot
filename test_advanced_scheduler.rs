use course_pilot::{AdvancedSchedulerSettings, DistributionStrategy, DifficultyLevel};

fn main() {
    // Test the new types and their methods
    println!("Testing Advanced Scheduler Integration...");

    // Test DistributionStrategy
    let strategies = DistributionStrategy::all();
    println!("Available strategies: {:?}", strategies);
    
    for strategy in &strategies {
        println!("- {}: {}", strategy.display_name(), strategy.description());
    }

    // Test DifficultyLevel
    let levels = DifficultyLevel::all();
    println!("\nAvailable difficulty levels: {:?}", levels);
    
    for level in &levels {
        println!("- {}", level.display_name());
    }

    // Test AdvancedSchedulerSettings
    let default_settings = AdvancedSchedulerSettings::default();
    println!("\nDefault advanced settings: {:?}", default_settings);

    // Test custom settings
    let custom_settings = AdvancedSchedulerSettings {
        strategy: DistributionStrategy::SpacedRepetition,
        difficulty_adaptation: true,
        spaced_repetition_enabled: true,
        cognitive_load_balancing: false,
        user_experience_level: DifficultyLevel::Beginner,
        custom_intervals: Some(vec![1, 3, 7, 14, 30]),
    };
    println!("Custom settings: {:?}", custom_settings);

    println!("\nAdvanced Scheduler Integration test completed successfully!");
}