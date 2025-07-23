//! Unit tests for PlanExt trait and progress calculation

use chrono::Utc;
use uuid::Uuid;

use course_pilot::types::{Plan, PlanExt, PlanItem, PlanItemIdentifier, PlanSettings};

fn create_test_plan() -> Plan {
    let mut plan = Plan::new(
        Uuid::new_v4(),
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
        },
    );

    // Add test plan items
    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module 1".to_string(),
        section_title: "Section 1".to_string(),
        video_indices: vec![0],
        completed: true,
    });

    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module 1".to_string(),
        section_title: "Section 2".to_string(),
        video_indices: vec![1],
        completed: false,
    });

    plan.items.push(PlanItem {
        date: Utc::now(),
        module_title: "Module 2".to_string(),
        section_title: "Section 1".to_string(),
        video_indices: vec![2],
        completed: true,
    });

    plan
}

#[test]
fn test_plan_item_identifier() {
    let plan = create_test_plan();

    let identifier = plan.get_item_identifier(0);
    assert_eq!(identifier.plan_id, plan.id);
    assert_eq!(identifier.item_index, 0);

    let identifier2 = PlanItemIdentifier::new(plan.id, 1);
    assert_eq!(identifier2.plan_id, plan.id);
    assert_eq!(identifier2.item_index, 1);
}

#[test]
fn test_update_item_completion() {
    let mut plan = create_test_plan();

    // Test valid update
    assert!(plan.items[1].completed == false);
    let result = plan.update_item_completion(1, true);
    assert!(result.is_ok());
    assert!(plan.items[1].completed == true);

    // Test toggle back
    let result = plan.update_item_completion(1, false);
    assert!(result.is_ok());
    assert!(plan.items[1].completed == false);
}

#[test]
fn test_update_item_completion_out_of_bounds() {
    let mut plan = create_test_plan();

    // Test out of bounds
    let result = plan.update_item_completion(999, true);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_calculate_progress() {
    let plan = create_test_plan();

    let (completed_count, total_count, percentage) = plan.calculate_progress();

    assert_eq!(completed_count, 2); // 2 items are completed
    assert_eq!(total_count, 3); // 3 total items
    assert_eq!(percentage, 66.66667); // 2/3 * 100
}

#[test]
fn test_calculate_progress_empty_plan() {
    let plan = Plan::new(
        Uuid::new_v4(),
        PlanSettings {
            start_date: Utc::now(),
            sessions_per_week: 3,
            session_length_minutes: 60,
            include_weekends: false,
        },
    );

    let (completed_count, total_count, percentage) = plan.calculate_progress();

    assert_eq!(completed_count, 0);
    assert_eq!(total_count, 0);
    assert_eq!(percentage, 0.0);
}

#[test]
fn test_calculate_progress_all_completed() {
    let mut plan = create_test_plan();

    // Mark all items as completed
    for item in &mut plan.items {
        item.completed = true;
    }

    let (completed_count, total_count, percentage) = plan.calculate_progress();

    assert_eq!(completed_count, 3);
    assert_eq!(total_count, 3);
    assert_eq!(percentage, 100.0);
}

#[test]
fn test_calculate_progress_none_completed() {
    let mut plan = create_test_plan();

    // Mark all items as not completed
    for item in &mut plan.items {
        item.completed = false;
    }

    let (completed_count, total_count, percentage) = plan.calculate_progress();

    assert_eq!(completed_count, 0);
    assert_eq!(total_count, 3);
    assert_eq!(percentage, 0.0);
}

#[test]
fn test_plan_item_identifier_equality() {
    let plan_id = Uuid::new_v4();

    let id1 = PlanItemIdentifier::new(plan_id, 0);
    let id2 = PlanItemIdentifier::new(plan_id, 0);
    let id3 = PlanItemIdentifier::new(plan_id, 1);
    let id4 = PlanItemIdentifier::new(Uuid::new_v4(), 0);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
    assert_ne!(id1, id4);
}
