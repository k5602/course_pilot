//! Advanced course structure analysis processor
//!
//! This module implements sophisticated course structure analysis using the full potential
//! of the clustering infrastructure: TF-IDF analysis, K-means clustering, duration balancing,
//! topic extraction, and multi-pass optimization for optimal course organization.

use crate::NlpError;
use crate::nlp::clustering::{
    BalancedCluster, ContentAnalysis, ContentClusterer, DurationBalancer, EnsembleMethod,
    HierarchicalClusterer, HybridClusterer, KMeansClusterer, LdaClusterer, OptimizedCluster,
    StrategySelection, TfIdfAnalyzer, TopicExtractor, VideoCluster, VideoWithMetadata,
};
use crate::nlp::{extract_numbers, is_module_indicator, normalize_text};
use crate::types::{
    ClusteringMetadata, ClusteringStrategy, CourseStructure, DifficultyLevel, Module, PlanSettings,
    Section, StructureMetadata, TopicInfo,
};
use regex::Regex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Structure a course from raw video titles using intelligent clustering
///
/// # Arguments
/// * `titles` - Vector of raw video titles to analyze
///
/// # Returns
/// * `Ok(CourseStructure)` - Structured course with modules and sections
/// * `Err(NlpError)` - Error if structuring fails
pub fn structure_course(titles: Vec<String>) -> Result<CourseStructure, NlpError> {
    if titles.is_empty() {
        return Err(NlpError::InvalidInput("No titles provided".to_string()));
    }

    let start_time = Instant::now();

    // Step 1: Check if we have explicit module structure that should be preserved
    let has_explicit_modules = titles.iter().any(|title| is_module_indicator(title));

    // Step 2: If we have explicit modules or insufficient content, use rule-based approach
    if has_explicit_modules || titles.len() < 8 {
        log::info!("Using rule-based approach due to explicit modules or small dataset");
        return structure_course_fallback(titles);
    }

    // Step 3: Try intelligent clustering approach for larger, unstructured datasets
    match structure_course_with_clustering(&titles) {
        Ok(clustered_structure) => {
            log::info!(
                "Successfully structured course using clustering in {}ms",
                start_time.elapsed().as_millis()
            );
            Ok(clustered_structure)
        }
        Err(clustering_error) => {
            log::warn!(
                "Clustering failed: {clustering_error}, falling back to rule-based approach"
            );

            // Step 4: Fallback to existing rule-based approach
            structure_course_fallback(titles)
        }
    }
}

/// Structure course using advanced clustering pipeline with full optimization
fn structure_course_with_clustering(titles: &[String]) -> Result<CourseStructure, NlpError> {
    let start_time = Instant::now();

    // For small datasets (< 5 titles), clustering is not effective, so fail early
    if titles.len() < 5 {
        return Err(NlpError::Processing(
            "Insufficient content for clustering".to_string(),
        ));
    }

    // Step 1: Advanced content analysis with custom parameters
    let content_analysis = perform_advanced_content_analysis(titles)?;
    let clustering_strategy = select_optimal_clustering_strategy(&content_analysis);

    // Step 2: Apply full clustering pipeline based on selected strategy
    let (modules, clustering_metadata) = match clustering_strategy {
        ClusteringStrategy::ContentBased => apply_advanced_content_clustering(titles)?,
        ClusteringStrategy::DurationBased => apply_advanced_duration_clustering(titles)?,
        ClusteringStrategy::Hierarchical => apply_hierarchical_clustering(titles)?,
        ClusteringStrategy::Lda => apply_lda_clustering(titles)?,
        ClusteringStrategy::Hybrid => apply_advanced_hybrid_clustering_v2(titles)?,
        ClusteringStrategy::Fallback => {
            return Err(NlpError::Processing(
                "Clustering strategy selection failed".to_string(),
            ));
        }
    };

    // Step 3: Generate enhanced metadata with quality assessment
    let mut metadata = generate_enhanced_metadata(titles, &modules, &content_analysis);
    metadata.structure_quality_score = Some(clustering_metadata.quality_score);
    metadata.content_coherence_score = Some(calculate_advanced_coherence_score(&modules));

    // Step 4: Create final structure with comprehensive clustering metadata
    let mut final_clustering_metadata = clustering_metadata;
    final_clustering_metadata.processing_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(CourseStructure::new_with_clustering(
        modules,
        metadata,
        final_clustering_metadata,
    ))
}

/// Fallback to rule-based course structuring
fn structure_course_fallback(titles: Vec<String>) -> Result<CourseStructure, NlpError> {
    // Step 1: Analyze titles to identify structure patterns
    let analysis = analyze_title_patterns(&titles)?;

    // Step 2: Choose the best structuring strategy based on analysis
    let strategy = choose_structuring_strategy(&analysis);

    // Step 3: Apply the chosen strategy to create structure
    let modules = match strategy {
        StructuringStrategy::Hierarchical => create_hierarchical_structure(&titles, &analysis)?,
        StructuringStrategy::Sequential => create_sequential_structure(&titles, &analysis)?,
        StructuringStrategy::Thematic => create_thematic_structure(&titles, &analysis)?,
        StructuringStrategy::Fallback => create_fallback_structure(&titles)?,
    };

    // Step 4: Generate metadata
    let metadata = generate_enhanced_metadata(
        &titles,
        &modules,
        &create_basic_analysis_for_difficulty(&titles),
    );

    Ok(CourseStructure::new_basic(modules, metadata))
}

/// Analysis results for title patterns
#[derive(Debug)]
#[allow(dead_code)]
struct TitleAnalysis {
    has_numeric_sequence: bool,
    has_explicit_modules: bool,
    has_consistent_naming: bool,
    module_boundaries: Vec<usize>,
    estimated_difficulty: DifficultyLevel,
}

/// Advanced content analysis results with comprehensive metrics
#[derive(Debug)]
struct AdvancedContentAnalysis {
    content_analysis: ContentAnalysis,
    // TODO: Integrate later - Could be used for topic-based session grouping
    #[allow(dead_code)]
    extracted_topics: Vec<TopicInfo>,
    content_diversity_score: f32,
    // TODO: Integrate later - Could be used for similarity-based optimization
    #[allow(dead_code)]
    title_similarity_score: f32,
    has_clear_topics: bool,
    // TODO: Integrate later - Could be used for dynamic cluster count selection
    #[allow(dead_code)]
    estimated_optimal_clusters: usize,
    content_complexity: f32,
    duration_variance: f32,
    clustering_feasibility: f32,
    vocabulary_richness: f32,
    topic_coherence_score: f32,
}

/// Available structuring strategies
#[derive(Debug)]
enum StructuringStrategy {
    Hierarchical, // Clear module/section hierarchy
    Sequential,   // Linear progression with grouping
    Thematic,     // Topic-based grouping
    Fallback,     // Simple chunking when no pattern is clear
}

/// Analyze title patterns to understand course structure
fn analyze_title_patterns(titles: &[String]) -> Result<TitleAnalysis, NlpError> {
    let mut module_boundaries = Vec::new();

    // Check for numeric sequences
    let mut numeric_titles = 0;
    for title in titles {
        let numbers = extract_numbers(title);
        if !numbers.is_empty() {
            numeric_titles += 1;
        }
    }
    let has_numeric_sequence = numeric_titles > titles.len() / 2;

    // Check for explicit module indicators
    let mut module_indicators = 0;
    for (i, title) in titles.iter().enumerate() {
        if is_module_indicator(title) {
            module_indicators += 1;
            module_boundaries.push(i);
        }
    }
    let has_explicit_modules = module_indicators > 0;

    // Check for consistent naming patterns
    let patterns = find_naming_patterns(titles);
    let has_consistent_naming = patterns.len() > 1 && patterns.values().any(|&count| count > 2);

    // Estimate difficulty based on vocabulary complexity
    let estimated_difficulty = estimate_difficulty(titles);

    Ok(TitleAnalysis {
        has_numeric_sequence,
        has_explicit_modules,
        has_consistent_naming,
        module_boundaries,
        estimated_difficulty,
    })
}

/// Find common naming patterns in titles
fn find_naming_patterns(titles: &[String]) -> HashMap<String, usize> {
    let mut patterns = HashMap::new();
    let pattern_regex = Regex::new(r"^([a-zA-Z\s]+)\s*\d+").unwrap();

    for title in titles {
        if let Some(captures) = pattern_regex.captures(title) {
            if let Some(pattern) = captures.get(1) {
                let normalized_pattern = normalize_text(pattern.as_str());
                *patterns.entry(normalized_pattern).or_insert(0) += 1;
            }
        }
    }

    patterns
}

/// Estimate the difficulty level of the course
fn estimate_difficulty(titles: &[String]) -> DifficultyLevel {
    let beginner_keywords = [
        "introduction",
        "basics",
        "fundamentals",
        "getting started",
        "beginner",
    ];
    let advanced_keywords = [
        "advanced",
        "expert",
        "master",
        "deep dive",
        "optimization",
        "architecture",
    ];

    let mut beginner_count = 0;
    let mut advanced_count = 0;

    for title in titles {
        let title_lower = title.to_lowercase();

        for keyword in &beginner_keywords {
            if title_lower.contains(keyword) {
                beginner_count += 1;
                break;
            }
        }

        for keyword in &advanced_keywords {
            if title_lower.contains(keyword) {
                advanced_count += 1;
                break;
            }
        }
    }

    match (beginner_count, advanced_count) {
        (b, a) if b > a && b > titles.len() / 4 => DifficultyLevel::Beginner,
        (b, a) if a > b && a > titles.len() / 4 => DifficultyLevel::Advanced,
        (b, a) if b > 0 && a > 0 => DifficultyLevel::Intermediate, // Mixed content defaults to intermediate
        _ => DifficultyLevel::Intermediate,
    }
}

/// Choose the best structuring strategy based on analysis
fn choose_structuring_strategy(analysis: &TitleAnalysis) -> StructuringStrategy {
    if analysis.has_explicit_modules && analysis.module_boundaries.len() > 1 {
        StructuringStrategy::Hierarchical
    } else if analysis.has_numeric_sequence && analysis.has_consistent_naming {
        StructuringStrategy::Sequential
    } else if analysis.has_consistent_naming {
        StructuringStrategy::Thematic
    } else {
        StructuringStrategy::Fallback
    }
}

/// Create hierarchical structure based on explicit module indicators
fn create_hierarchical_structure(
    titles: &[String],
    _analysis: &TitleAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let mut modules = Vec::new();
    let mut current_sections = Vec::new();
    let mut current_module_title = "Introduction".to_string();

    for (i, title) in titles.iter().enumerate() {
        if is_module_indicator(title) {
            // If we have accumulated sections, save them as a module
            if !current_sections.is_empty() {
                modules.push(Module::new_basic(
                    current_module_title.clone(),
                    std::mem::take(&mut current_sections),
                ));
            }
            // Set new module title but don't add the module indicator as a section
            current_module_title = extract_module_title(title);
        } else {
            // Add non-module titles as sections
            current_sections.push(Section {
                title: title.clone(),
                video_index: i,
                duration: estimate_video_duration(title)
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            });
        }
    }

    // Add the last module
    if !current_sections.is_empty() {
        modules.push(Module::new_basic(
            current_module_title,
            current_sections.clone(),
        ));
    }

    Ok(modules)
}

/// Create sequential structure with natural grouping
fn create_sequential_structure(
    titles: &[String],
    _analysis: &TitleAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let chunk_size = calculate_optimal_chunk_size(titles.len());
    let mut modules = Vec::new();

    for (module_index, chunk) in titles.chunks(chunk_size).enumerate() {
        let module_title = generate_sequential_module_title(module_index + 1, chunk);
        let sections: Vec<Section> = chunk
            .iter()
            .enumerate()
            .map(|(section_index, title)| Section {
                title: title.clone(),
                video_index: module_index * chunk_size + section_index,
                duration: estimate_video_duration(title)
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            })
            .collect();

        modules.push(Module::new_basic(module_title, sections.clone()));
    }

    Ok(modules)
}

/// Create thematic structure based on content similarity
fn create_thematic_structure(
    titles: &[String],
    _analysis: &TitleAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let themes = identify_themes(titles)?;
    let mut modules = Vec::new();

    for (theme_name, video_indices) in themes {
        let sections: Vec<Section> = video_indices
            .into_iter()
            .map(|index| Section {
                title: titles[index].clone(),
                video_index: index,
                duration: estimate_video_duration(&titles[index])
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            })
            .collect();

        modules.push(Module::new_basic(theme_name, sections.clone()));
    }

    Ok(modules)
}

/// Create fallback structure with simple chunking
fn create_fallback_structure(titles: &[String]) -> Result<Vec<Module>, NlpError> {
    let chunk_size = 8; // Default chunk size for fallback
    let mut modules = Vec::new();

    for (module_index, chunk) in titles.chunks(chunk_size).enumerate() {
        let module_title = format!("Part {}", module_index + 1);
        let sections: Vec<Section> = chunk
            .iter()
            .enumerate()
            .map(|(section_index, title)| Section {
                title: title.clone(),
                video_index: module_index * chunk_size + section_index,
                duration: estimate_video_duration(title)
                    .unwrap_or_else(|| std::time::Duration::from_secs(0)),
            })
            .collect();

        modules.push(Module::new_basic(module_title, sections.clone()));
    }

    Ok(modules)
}

/// Extract a clean module title from a title with module indicators
fn extract_module_title(title: &str) -> String {
    let title_clean = title
        .split(':')
        .next()
        .unwrap_or(title)
        .split('-')
        .next()
        .unwrap_or(title)
        .trim();

    if title_clean.is_empty() {
        "Untitled Module".to_string()
    } else {
        title_clean.to_string()
    }
}

/// Calculate optimal chunk size for sequential structuring
fn calculate_optimal_chunk_size(total_videos: usize) -> usize {
    match total_videos {
        1..=20 => std::cmp::max(total_videos / 3, 1),
        21..=50 => total_videos / 5,
        51..=100 => total_videos / 7,
        _ => total_videos / 10,
    }
}

/// Generate a module title for sequential structure
fn generate_sequential_module_title(module_number: usize, sections: &[String]) -> String {
    // Try to extract common theme from section titles
    let first_title = &sections[0];
    let words: Vec<&str> = first_title.split_whitespace().collect();

    if words.len() > 1 {
        let theme = words[0..std::cmp::min(2, words.len())].join(" ");
        format!("Module {module_number}: {theme}")
    } else {
        format!("Module {module_number}")
    }
}

/// Identify themes in titles using clustering
fn identify_themes(titles: &[String]) -> Result<Vec<(String, Vec<usize>)>, NlpError> {
    let mut themes = Vec::new();
    let mut used_indices = std::collections::HashSet::new();

    // Simple keyword-based clustering
    let keywords = extract_common_keywords(titles);

    for keyword in keywords {
        let mut theme_indices = Vec::new();

        for (i, title) in titles.iter().enumerate() {
            if !used_indices.contains(&i) && title.to_lowercase().contains(&keyword) {
                theme_indices.push(i);
                used_indices.insert(i);
            }
        }

        if theme_indices.len() > 1 {
            let theme_name = keyword
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect::<String>() + &keyword[1..])
                .unwrap_or_else(|| keyword.clone());

            themes.push((theme_name, theme_indices));
        }
    }

    // Handle remaining uncategorized titles
    let remaining_indices: Vec<usize> = (0..titles.len())
        .filter(|i| !used_indices.contains(i))
        .collect();

    if !remaining_indices.is_empty() {
        themes.push(("Miscellaneous".to_string(), remaining_indices));
    }

    // If no themes found, create single theme
    if themes.is_empty() {
        themes.push(("Course Content".to_string(), (0..titles.len()).collect()));
    }

    Ok(themes)
}

/// Extract common keywords from titles
fn extract_common_keywords(titles: &[String]) -> Vec<String> {
    let mut word_counts = HashMap::new();

    for title in titles {
        for word in normalize_text(title).split_whitespace() {
            if word.len() > 3 {
                // Only consider words longer than 3 characters
                *word_counts.entry(word.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut keywords: Vec<_> = word_counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect();

    keywords.sort_by(|a, b| b.1.cmp(&a.1));
    keywords.into_iter().map(|(word, _)| word).take(5).collect()
}

/// Estimate video duration based on title content
fn estimate_video_duration(title: &str) -> Option<Duration> {
    // Simple heuristic based on title length and keywords
    let _base_duration = Duration::from_secs(600); // 10 minutes default

    let duration_minutes = if title.to_lowercase().contains("introduction") {
        5 // Shorter for introductions
    } else if title.to_lowercase().contains("project") || title.to_lowercase().contains("exercise")
    {
        20 // Longer for practical work
    } else {
        10 // Default
    };

    Some(Duration::from_secs(duration_minutes * 60))
}

/// Perform advanced content analysis with sophisticated TF-IDF and topic extraction
fn perform_advanced_content_analysis(
    titles: &[String],
) -> Result<AdvancedContentAnalysis, NlpError> {
    if titles.len() < 5 {
        return Err(NlpError::InvalidInput(
            "Insufficient content for clustering analysis".to_string(),
        ));
    }

    // Step 1: Configure TF-IDF analyzer with content-aware parameters
    let analyzer = configure_tfidf_analyzer(titles);
    let content_analysis = analyzer
        .analyze_content(titles)
        .map_err(|e| NlpError::Processing(format!("Content analysis failed: {e}")))?;

    // Step 2: Extract topics using advanced topic extractor
    let topic_extractor = TopicExtractor::new(2, 0.15); // Adjusted thresholds
    let topic_keywords: HashMap<String, f32> = content_analysis
        .topic_keywords
        .iter()
        .enumerate()
        .map(|(i, keyword)| (keyword.clone(), 1.0 - (i as f32 * 0.1)))
        .collect();
    let clustering_topics = topic_extractor.extract_topics(titles, &topic_keywords);

    // Convert clustering TopicInfo to types TopicInfo
    let extracted_topics: Vec<crate::types::TopicInfo> = clustering_topics
        .into_iter()
        .map(|topic| crate::types::TopicInfo {
            keyword: topic.keyword,
            relevance_score: topic.relevance_score,
            video_count: topic.related_videos.len(),
        })
        .collect();

    // Step 3: Calculate advanced content metrics
    let avg_similarity = content_analysis.similarity_matrix.average_similarity();
    let content_diversity_score = 1.0 - avg_similarity;
    let title_similarity_score = avg_similarity;
    let has_clear_topics =
        extracted_topics.len() >= 2 && extracted_topics.iter().any(|t| t.relevance_score > 0.5);

    // Step 4: Estimate optimal clustering parameters
    let estimated_optimal_clusters =
        calculate_optimal_cluster_count(titles.len(), &content_analysis);
    let content_complexity = calculate_content_complexity(&content_analysis, titles);
    let duration_variance = estimate_duration_variance_from_titles(titles);

    // Step 5: Assess clustering feasibility
    let clustering_feasibility = assess_clustering_feasibility(
        content_diversity_score,
        title_similarity_score,
        has_clear_topics,
        titles.len(),
    );

    let vocabulary_richness = content_analysis.vocabulary.len() as f32 / titles.len() as f32;
    let topic_coherence_score = calculate_topic_coherence(&extracted_topics);

    Ok(AdvancedContentAnalysis {
        content_analysis,
        extracted_topics,
        content_diversity_score,
        title_similarity_score,
        has_clear_topics,
        estimated_optimal_clusters,
        content_complexity,
        duration_variance,
        clustering_feasibility,
        vocabulary_richness,
        topic_coherence_score,
    })
}

/// Select optimal clustering strategy using advanced analysis metrics
fn select_optimal_clustering_strategy(analysis: &AdvancedContentAnalysis) -> ClusteringStrategy {
    // Calculate weighted strategy scores
    let content_score = calculate_content_strategy_score(analysis);
    let duration_score = calculate_duration_strategy_score(analysis);
    let hybrid_score = calculate_hybrid_strategy_score(analysis);

    log::info!(
        "Strategy scores - Content: {content_score:.3}, Duration: {duration_score:.3}, Hybrid: {hybrid_score:.3}"
    );

    // Calculate additional scores for new algorithms
    let hierarchical_score = calculate_hierarchical_strategy_score(analysis);
    let lda_score = calculate_lda_strategy_score(analysis);

    log::info!(
        "Extended strategy scores - Content: {content_score:.3}, Duration: {duration_score:.3}, Hybrid: {hybrid_score:.3}, Hierarchical: {hierarchical_score:.3}, LDA: {lda_score:.3}"
    );

    // Select strategy with highest score, with minimum feasibility threshold
    let scores = [
        (ClusteringStrategy::ContentBased, content_score),
        (ClusteringStrategy::DurationBased, duration_score),
        (ClusteringStrategy::Hierarchical, hierarchical_score),
        (ClusteringStrategy::Lda, lda_score),
        (ClusteringStrategy::Hybrid, hybrid_score),
    ];

    // Find the strategy with the highest score above threshold
    let best_strategy = scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(strategy, score)| (strategy.clone(), *score));

    match best_strategy {
        Some((strategy, score)) if score > 0.4 => strategy,
        _ => {
            log::warn!("All clustering strategies scored low, using fallback");
            ClusteringStrategy::Fallback
        }
    }
}

/// Apply advanced content-based clustering with full optimization pipeline
fn apply_advanced_content_clustering(
    titles: &[String],
) -> Result<(Vec<Module>, ClusteringMetadata), NlpError> {
    let start_time = Instant::now();

    // Step 1: Configure advanced TF-IDF analyzer
    let analyzer = configure_tfidf_analyzer(titles);
    let content_analysis = analyzer
        .analyze_content(titles)
        .map_err(|e| NlpError::Processing(format!("Content analysis failed: {e}")))?;

    // Step 2: Configure K-means clusterer with optimized parameters
    let clusterer = configure_kmeans_clusterer(&content_analysis);
    let optimal_k = clusterer.determine_optimal_k(&content_analysis.feature_vectors);

    log::info!(
        "Determined optimal K: {} for {} videos",
        optimal_k,
        titles.len()
    );

    // Step 3: Perform clustering with quality assessment
    let video_clusters = clusterer
        .cluster_videos(&content_analysis, optimal_k)
        .map_err(|e| NlpError::Processing(format!("Clustering failed: {e}")))?;

    // Step 4: Optimize clusters with duration constraints
    let durations: Vec<Duration> = titles
        .iter()
        .map(|title| estimate_video_duration(title).unwrap_or_else(|| Duration::from_secs(600)))
        .collect();

    let optimized_clusters = clusterer
        .optimize_clusters(video_clusters.clone(), &durations)
        .map_err(|e| NlpError::Processing(format!("Cluster optimization failed: {e}")))?;

    // Step 5: Apply duration balancing with advanced bin packing
    let default_settings = create_default_plan_settings();
    let duration_balancer = DurationBalancer::from_plan_settings(&default_settings);
    let balanced_clusters = duration_balancer
        .balance_clusters(optimized_clusters)
        .map_err(|e| NlpError::Processing(format!("Duration balancing failed: {e}")))?;

    // Step 6: Extract topics and generate intelligent cluster names
    let topic_extractor = TopicExtractor::new(2, 0.15);
    let topic_keywords: HashMap<String, f32> = content_analysis
        .topic_keywords
        .iter()
        .enumerate()
        .map(|(i, keyword)| (keyword.clone(), 1.0 - (i as f32 * 0.1)))
        .collect();
    let clustering_topics = topic_extractor.extract_topics(titles, &topic_keywords);

    // Convert clustering TopicInfo to types TopicInfo
    let extracted_topics: Vec<crate::types::TopicInfo> = clustering_topics
        .into_iter()
        .map(|topic| crate::types::TopicInfo {
            keyword: topic.keyword,
            relevance_score: topic.relevance_score,
            video_count: topic.related_videos.len(),
        })
        .collect();

    // Step 7: Convert balanced clusters to modules with intelligent naming
    let modules = convert_balanced_clusters_to_modules(
        balanced_clusters.clone(),
        &extracted_topics,
        &topic_extractor,
        titles,
    )?;

    // Step 8: Calculate comprehensive quality metrics
    let quality_metrics =
        calculate_comprehensive_quality_metrics(&video_clusters, &modules, &content_analysis);

    // Step 9: Create enhanced clustering metadata with confidence scoring and rationale
    let sections: Vec<Section> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| Section {
            title: title.clone(),
            video_index: i,
            duration: estimate_video_duration(title).unwrap_or_else(|| Duration::from_secs(600)),
        })
        .collect();

    let input_metrics =
        crate::nlp::clustering::metadata_generator::InputMetricsCalculator::calculate_metrics(
            &sections,
        );
    let performance_metrics = crate::nlp::clustering::PerformanceMetrics {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        content_analysis_time_ms: (start_time.elapsed().as_millis() / 3) as u64, // Estimate
        clustering_time_ms: (start_time.elapsed().as_millis() / 3) as u64,       // Estimate
        optimization_time_ms: (start_time.elapsed().as_millis() / 3) as u64,     // Estimate
        peak_memory_usage_bytes: 1024 * 1024, // Simplified estimate
        algorithm_iterations: optimal_k as u32,
        input_metrics,
    };

    let clustering_metadata = crate::nlp::clustering::metadata_generator::MetadataGenerator::generate_complete_metadata_from_balanced(
        &sections,
        &balanced_clusters,
        crate::types::ClusteringAlgorithm::KMeans,
        crate::types::ClusteringStrategy::ContentBased,
        analyzer.min_similarity_threshold,
        extracted_topics,
        performance_metrics,
    );

    log::info!(
        "Content clustering completed: {} modules, quality: {:.3}, time: {}ms",
        modules.len(),
        quality_metrics.overall_quality,
        clustering_metadata.processing_time_ms
    );

    Ok((modules, clustering_metadata))
}

/// Apply advanced duration-based clustering with sophisticated balancing
fn apply_advanced_duration_clustering(
    titles: &[String],
) -> Result<(Vec<Module>, ClusteringMetadata), NlpError> {
    let start_time = Instant::now();

    // Step 1: Estimate video durations with improved heuristics
    let video_metadata = create_video_metadata_from_titles(titles);

    // Step 2: Apply advanced duration balancing
    let default_settings = create_default_plan_settings();
    let duration_balancer = DurationBalancer::from_plan_settings(&default_settings);

    // Step 3: Create initial clusters based on duration patterns
    let initial_clusters =
        create_duration_based_initial_clusters(&video_metadata, &default_settings)?;

    // Step 4: Apply bin packing optimization
    let balanced_clusters = duration_balancer
        .balance_clusters(initial_clusters)
        .map_err(|e| NlpError::Processing(format!("Duration balancing failed: {e}")))?;

    // Step 5: Extract basic topics for naming (even in duration-based approach)
    let topic_extractor = TopicExtractor::new(1, 0.1); // Lower thresholds for duration-based
    let basic_tfidf_scores = calculate_basic_tfidf_scores(titles);
    let clustering_topics = topic_extractor.extract_topics(titles, &basic_tfidf_scores);

    // Convert clustering TopicInfo to types TopicInfo
    let extracted_topics: Vec<crate::types::TopicInfo> = clustering_topics
        .into_iter()
        .map(|topic| crate::types::TopicInfo {
            keyword: topic.keyword,
            relevance_score: topic.relevance_score,
            video_count: topic.related_videos.len(),
        })
        .collect();

    // Step 6: Convert to modules with duration-aware naming
    let modules = convert_balanced_clusters_to_modules_duration_focused(
        balanced_clusters.clone(),
        &extracted_topics,
        titles,
    )?;

    // Step 7: Calculate duration-focused quality metrics
    let quality_score = calculate_duration_quality_score(&modules);

    // Step 8: Create clustering metadata with confidence scoring and rationale
    let sections: Vec<Section> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| Section {
            title: title.clone(),
            video_index: i,
            duration: estimate_video_duration(title).unwrap_or_else(|| Duration::from_secs(600)),
        })
        .collect();

    let input_metrics =
        crate::nlp::clustering::metadata_generator::InputMetricsCalculator::calculate_metrics(
            &sections,
        );
    let performance_metrics = crate::nlp::clustering::PerformanceMetrics {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        content_analysis_time_ms: (start_time.elapsed().as_millis() / 4) as u64, // Estimate
        clustering_time_ms: (start_time.elapsed().as_millis() / 2) as u64, // Duration clustering is simpler
        optimization_time_ms: (start_time.elapsed().as_millis() / 4) as u64, // Estimate
        peak_memory_usage_bytes: 512 * 1024, // Lower memory usage for duration-based
        algorithm_iterations: 1,             // Duration clustering doesn't iterate
        input_metrics,
    };

    let clustering_metadata = crate::nlp::clustering::metadata_generator::MetadataGenerator::generate_complete_metadata_from_balanced(
        &sections,
        &balanced_clusters,
        crate::types::ClusteringAlgorithm::Hybrid, // Duration balancing uses hybrid approach
        crate::types::ClusteringStrategy::DurationBased,
        0.3, // Lower threshold for duration-based
        extracted_topics,
        performance_metrics,
    );

    log::info!(
        "Duration clustering completed: {} modules, quality: {:.3}, time: {}ms",
        modules.len(),
        quality_score,
        clustering_metadata.processing_time_ms
    );

    Ok((modules, clustering_metadata))
}

// REMOVED: apply_advanced_hybrid_clustering - superseded by apply_advanced_hybrid_clustering_v2

// TODO: Integrate later - This function could be used for post-processing module optimization
/// Balance modules by duration constraints
#[allow(dead_code)]
fn balance_modules_by_duration(modules: &mut Vec<Module>) {
    // Simple duration balancing - split oversized modules
    let target_duration = Duration::from_secs(3600); // 1 hour target
    let mut i = 0;

    while i < modules.len() {
        if modules[i].total_duration > target_duration * 2 {
            // Split this module
            let module = modules.remove(i);
            let split_modules = split_module_by_duration(module, target_duration);

            // Insert split modules
            for (j, split_module) in split_modules.into_iter().enumerate() {
                modules.insert(i + j, split_module);
            }

            i += 2; // Skip the newly inserted modules
        } else {
            i += 1;
        }
    }
}

// TODO: Integrate later - This function could be used for dynamic module splitting
/// Split a module into smaller duration-balanced modules
#[allow(dead_code)]
fn split_module_by_duration(module: Module, target_duration: Duration) -> Vec<Module> {
    let mut result = Vec::new();
    let mut current_sections = Vec::new();
    let mut current_duration = Duration::from_secs(0);
    let mut part_number = 1;

    // Clone the fields we'll need to reuse
    let module_title = module.title.clone();
    let similarity_score = module.similarity_score.unwrap_or(0.5);
    let topic_keywords = module.topic_keywords.clone();
    let difficulty_level = module
        .difficulty_level
        .unwrap_or(DifficultyLevel::Intermediate);

    for section in module.sections {
        if current_duration + section.duration > target_duration && !current_sections.is_empty() {
            // Create a new module from current sections
            let part_title = format!("{module_title} - Part {part_number}");
            let part_module = Module::new_with_clustering(
                part_title,
                current_sections,
                similarity_score,
                topic_keywords.clone(),
                difficulty_level,
            );
            result.push(part_module);

            current_sections = Vec::new();
            current_duration = Duration::from_secs(0);
            part_number += 1;
        }

        current_duration += section.duration;
        current_sections.push(section);
    }

    // Handle remaining sections
    if !current_sections.is_empty() {
        let part_title = if part_number == 1 {
            module_title.clone()
        } else {
            format!("{module_title} - Part {part_number}")
        };

        let final_module = Module::new_with_clustering(
            part_title,
            current_sections,
            similarity_score,
            topic_keywords.clone(),
            difficulty_level,
        );
        result.push(final_module);
    }

    if result.is_empty() {
        vec![Module::new_with_clustering(
            module_title,
            Vec::new(),
            similarity_score,
            topic_keywords,
            difficulty_level,
        )] // Return a reconstructed module if splitting failed
    } else {
        result
    }
}

/// Calculate clustering quality score from video clusters
fn calculate_clustering_quality_score(clusters: &[crate::nlp::clustering::VideoCluster]) -> f32 {
    if clusters.is_empty() {
        return 0.0;
    }

    let avg_similarity: f32 =
        clusters.iter().map(|c| c.similarity_score).sum::<f32>() / clusters.len() as f32;

    // Normalize to 0-1 range
    avg_similarity.clamp(0.0, 1.0)
}

// TODO: Integrate later - This function could be used for quality assessment
/// Calculate content coherence score for modules
#[allow(dead_code)]
fn calculate_content_coherence_score(modules: &[Module]) -> f32 {
    if modules.is_empty() {
        return 0.0;
    }

    let avg_similarity: f32 = modules
        .iter()
        .filter_map(|m| m.similarity_score)
        .sum::<f32>()
        / modules.len() as f32;

    avg_similarity.clamp(0.0, 1.0)
}

/// Generate enhanced metadata with comprehensive analysis
fn generate_enhanced_metadata(
    titles: &[String],
    modules: &[Module],
    analysis: &AdvancedContentAnalysis,
) -> StructureMetadata {
    let total_videos = titles.len();

    let estimated_duration_hours = modules
        .iter()
        .flat_map(|m| &m.sections)
        .map(|s| s.duration)
        .map(|d| d.as_secs_f32() / 3600.0)
        .sum::<f32>();

    let difficulty_level = match estimate_advanced_difficulty(titles, analysis) {
        DifficultyLevel::Beginner => Some("Beginner".to_string()),
        DifficultyLevel::Intermediate => Some("Intermediate".to_string()),
        DifficultyLevel::Advanced => Some("Advanced".to_string()),
        DifficultyLevel::Expert => Some("Expert".to_string()),
    };

    StructureMetadata {
        total_videos,
        total_duration: modules.iter().map(|m| m.total_duration).sum(),
        estimated_duration_hours: Some(estimated_duration_hours),
        difficulty_level,
        structure_quality_score: Some(analysis.clustering_feasibility),
        content_coherence_score: Some(analysis.topic_coherence_score),
    }
}

// ============================================================================
// ADVANCED CLUSTERING HELPER FUNCTIONS
// ============================================================================

/// Configure TF-IDF analyzer with content-aware parameters
fn configure_tfidf_analyzer(titles: &[String]) -> TfIdfAnalyzer {
    let vocab_size = estimate_vocabulary_size(titles);
    let content_complexity = estimate_content_complexity_from_titles(titles);

    // Adjust parameters based on content characteristics
    let similarity_threshold = if content_complexity > 0.7 { 0.7 } else { 0.6 };
    let max_features = std::cmp::min(vocab_size * 2, 2000);
    let min_term_frequency = if titles.len() > 50 { 3 } else { 2 };

    TfIdfAnalyzer::new(similarity_threshold, max_features, min_term_frequency)
}

/// Configure TF-IDF analyzer specifically for hybrid clustering
#[allow(dead_code)]
fn configure_tfidf_analyzer_for_hybrid(
    _titles: &[String],
    analysis: &AdvancedContentAnalysis,
) -> TfIdfAnalyzer {
    // Balance content and duration considerations
    let similarity_threshold = 0.55; // Slightly lower for hybrid
    let max_features = std::cmp::min(analysis.content_analysis.vocabulary.len() * 3, 1500);
    let min_term_frequency = if analysis.vocabulary_richness > 0.5 {
        3
    } else {
        2
    };

    TfIdfAnalyzer::new(similarity_threshold, max_features, min_term_frequency)
}

/// Configure K-means clusterer with optimized parameters
fn configure_kmeans_clusterer(content_analysis: &ContentAnalysis) -> KMeansClusterer {
    let max_iterations = if content_analysis.feature_vectors.len() > 100 {
        200
    } else {
        100
    };
    let convergence_threshold = 0.001;
    let random_seed = Some(42); // Reproducible results

    KMeansClusterer::new(max_iterations, convergence_threshold, random_seed)
}

/// Configure K-means clusterer for hybrid approach
#[allow(dead_code)]
fn configure_kmeans_clusterer_for_hybrid(
    _content_analysis: &ContentAnalysis,
    advanced_analysis: &AdvancedContentAnalysis,
) -> KMeansClusterer {
    let max_iterations = if advanced_analysis.content_complexity > 0.6 {
        150
    } else {
        100
    };
    let convergence_threshold = 0.0015; // Slightly more relaxed for hybrid
    let random_seed = Some(42);

    KMeansClusterer::new(max_iterations, convergence_threshold, random_seed)
}

/// Configure duration balancer for hybrid approach
#[allow(dead_code)]
fn configure_duration_balancer_for_hybrid(
    settings: &PlanSettings,
    analysis: &AdvancedContentAnalysis,
) -> DurationBalancer {
    let mut balancer = DurationBalancer::from_plan_settings(settings);

    // Adjust parameters based on content analysis
    if analysis.content_diversity_score > 0.7 {
        balancer.similarity_threshold = 0.5; // More flexible for diverse content
    }

    if analysis.duration_variance > 0.6 {
        balancer.max_duration_variance = 0.6; // Allow more variance for varied content
    }

    balancer
}

/// Calculate optimal cluster count using advanced heuristics
fn calculate_optimal_cluster_count(
    video_count: usize,
    content_analysis: &ContentAnalysis,
) -> usize {
    let base_clusters = match video_count {
        1..=10 => 2,
        11..=25 => video_count / 5,
        26..=50 => video_count / 6,
        51..=100 => video_count / 8,
        _ => video_count / 10,
    };

    // Adjust based on content diversity
    let diversity_factor = content_analysis.similarity_matrix.average_similarity();
    let adjusted_clusters = if diversity_factor < 0.3 {
        base_clusters + 1 // More clusters for diverse content
    } else if diversity_factor > 0.8 {
        std::cmp::max(base_clusters - 1, 2) // Fewer clusters for similar content
    } else {
        base_clusters
    };

    std::cmp::min(adjusted_clusters, video_count / 2)
}

/// Calculate content complexity from analysis results
fn calculate_content_complexity(content_analysis: &ContentAnalysis, titles: &[String]) -> f32 {
    let vocab_ratio = content_analysis.vocabulary.len() as f32 / titles.len() as f32;
    let avg_title_length =
        titles.iter().map(|t| t.len()).sum::<usize>() as f32 / titles.len() as f32;
    let length_factor = (avg_title_length / 50.0).clamp(0.1, 2.0); // Normalize around 50 chars

    (vocab_ratio * length_factor).clamp(0.0, 1.0)
}

/// Estimate duration variance from title characteristics
fn estimate_duration_variance_from_titles(titles: &[String]) -> f32 {
    let lengths: Vec<f32> = titles.iter().map(|t| t.len() as f32).collect();
    let mean = lengths.iter().sum::<f32>() / lengths.len() as f32;
    let variance = lengths.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / lengths.len() as f32;
    let std_dev = variance.sqrt();

    (std_dev / mean).clamp(0.0, 1.0)
}

/// Assess clustering feasibility based on content characteristics
fn assess_clustering_feasibility(
    diversity_score: f32,
    similarity_score: f32,
    has_clear_topics: bool,
    video_count: usize,
) -> f32 {
    let mut feasibility = 0.0;

    // Content diversity contributes to feasibility
    feasibility += diversity_score * 0.3;

    // Moderate similarity is good for clustering
    let similarity_factor = if similarity_score > 0.2 && similarity_score < 0.8 {
        1.0 - (similarity_score - 0.5).abs() * 2.0
    } else {
        0.3
    };
    feasibility += similarity_factor * 0.3;

    // Clear topics improve feasibility
    if has_clear_topics {
        feasibility += 0.25;
    }

    // Sufficient data improves feasibility
    let data_factor = match video_count {
        0..=4 => 0.0,
        5..=9 => 0.5,
        10..=19 => 0.8,
        _ => 1.0,
    };
    feasibility += data_factor * 0.15;

    feasibility.clamp(0.0, 1.0)
}

/// Calculate topic coherence score
fn calculate_topic_coherence(topics: &[TopicInfo]) -> f32 {
    if topics.is_empty() {
        return 0.0;
    }

    let avg_relevance = topics.iter().map(|t| t.relevance_score).sum::<f32>() / topics.len() as f32;
    let coverage_factor = (topics.len() as f32 / 10.0).clamp(0.1, 1.0); // Normalize around 10 topics

    (avg_relevance * coverage_factor).clamp(0.0, 1.0)
}

/// Calculate strategy scores for optimal selection
fn calculate_content_strategy_score(analysis: &AdvancedContentAnalysis) -> f32 {
    let mut score = 0.0;

    // High diversity favors content clustering
    score += analysis.content_diversity_score * 0.3;

    // Clear topics strongly favor content clustering
    if analysis.has_clear_topics {
        score += 0.3;
    }

    // Topic coherence improves content clustering
    score += analysis.topic_coherence_score * 0.2;

    // Vocabulary richness helps content clustering
    score += (analysis.vocabulary_richness * 0.5).clamp(0.0, 0.2);

    score.clamp(0.0, 1.0)
}

fn calculate_duration_strategy_score(analysis: &AdvancedContentAnalysis) -> f32 {
    let mut score = 0.0;

    // High duration variance favors duration clustering
    score += analysis.duration_variance * 0.4;

    // Low content diversity might favor duration approach
    score += (1.0 - analysis.content_diversity_score) * 0.2;

    // Low topic coherence might favor duration approach
    score += (1.0 - analysis.topic_coherence_score) * 0.2;

    // Base score for duration approach
    score += 0.2;

    score.clamp(0.0, 1.0)
}

fn calculate_hybrid_strategy_score(analysis: &AdvancedContentAnalysis) -> f32 {
    let content_score = calculate_content_strategy_score(analysis);
    let duration_score = calculate_duration_strategy_score(analysis);

    // Hybrid works well when both approaches have merit
    let balance_factor = 1.0 - (content_score - duration_score).abs();
    let base_score = (content_score + duration_score) / 2.0;

    (base_score * balance_factor + 0.1).clamp(0.0, 1.0)
}

/// Calculate hierarchical clustering strategy score
fn calculate_hierarchical_strategy_score(analysis: &AdvancedContentAnalysis) -> f32 {
    let mut score: f32 = 0.0;

    // Hierarchical works well with smaller datasets
    let total_videos = analysis.content_analysis.feature_vectors.len();
    let size_factor = if total_videos <= 50 {
        0.8
    } else if total_videos <= 100 {
        0.6
    } else {
        0.3
    };
    score += size_factor * 0.3;

    // Good for discovering natural hierarchies
    let hierarchy_factor = if analysis.content_diversity_score > 0.4 {
        0.9
    } else {
        0.5
    };
    score += hierarchy_factor * 0.3;

    // Works well when there's moderate similarity variance
    let variance_factor = if analysis.duration_variance > 0.15 && analysis.duration_variance < 0.4 {
        0.8
    } else {
        0.4
    };
    score += variance_factor * 0.4;

    score.clamp(0.0, 1.0)
}

/// Calculate LDA clustering strategy score
fn calculate_lda_strategy_score(analysis: &AdvancedContentAnalysis) -> f32 {
    let mut score: f32 = 0.0;

    // LDA works well with sufficient vocabulary diversity
    let vocab_factor = if analysis.vocabulary_richness > 0.3 {
        0.9
    } else if analysis.vocabulary_richness > 0.2 {
        0.6
    } else {
        0.2
    };
    score += vocab_factor * 0.4;

    // Good for text-rich content (use content complexity as proxy)
    let content_factor = if analysis.content_complexity > 0.6 {
        0.8
    } else if analysis.content_complexity > 0.4 {
        0.6
    } else {
        0.3
    };
    score += content_factor * 0.3;

    // Needs sufficient documents for topic modeling
    let total_videos = analysis.content_analysis.feature_vectors.len();
    let size_factor = if total_videos >= 10 {
        0.8
    } else if total_videos >= 6 {
        0.5
    } else {
        0.1
    };
    score += size_factor * 0.3;

    score.clamp(0.0, 1.0)
}

/// Create video metadata from titles with enhanced duration estimation
fn create_video_metadata_from_titles(titles: &[String]) -> Vec<VideoWithMetadata> {
    titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            VideoWithMetadata {
                index: i,
                title: title.clone(),
                duration: estimate_enhanced_video_duration(title),
                feature_vector: Default::default(), // Will be populated during analysis
                difficulty_score: estimate_difficulty_score_from_title(title),
                topic_tags: extract_basic_topic_tags(title),
            }
        })
        .collect()
}

/// Create enhanced video metadata with content analysis
fn create_enhanced_video_metadata(
    titles: &[String],
    content_analysis: &ContentAnalysis,
) -> Vec<VideoWithMetadata> {
    titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let feature_vector = content_analysis
                .feature_vectors
                .get(i)
                .cloned()
                .unwrap_or_default();

            VideoWithMetadata {
                index: i,
                title: title.clone(),
                duration: estimate_enhanced_video_duration(title),
                feature_vector: feature_vector.clone(),
                difficulty_score: estimate_difficulty_score_from_title(title),
                topic_tags: extract_topic_tags_from_features(&feature_vector),
            }
        })
        .collect()
}

/// Create default plan settings for clustering
fn create_default_plan_settings() -> PlanSettings {
    use chrono::Utc;

    PlanSettings {
        start_date: Utc::now(),
        sessions_per_week: 3,
        session_length_minutes: 60,
        include_weekends: false,
        advanced_settings: None,
    }
}

/// Estimate enhanced video duration with better heuristics
fn estimate_enhanced_video_duration(title: &str) -> Duration {
    let base_duration = 600; // 10 minutes
    let title_lower = title.to_lowercase();

    let duration_seconds =
        if title_lower.contains("introduction") || title_lower.contains("overview") {
            base_duration / 2 // 5 minutes for intros
        } else if title_lower.contains("project")
            || title_lower.contains("exercise")
            || title_lower.contains("hands-on")
        {
            base_duration * 2 // 20 minutes for practical work
        } else if title_lower.contains("deep dive")
            || title_lower.contains("advanced")
            || title_lower.contains("detailed")
        {
            (base_duration as f32 * 1.5) as u64 // 15 minutes for detailed content
        } else if title_lower.contains("quick")
            || title_lower.contains("brief")
            || title_lower.contains("summary")
        {
            base_duration / 3 // 3-4 minutes for quick content
        } else {
            // Adjust based on title length
            let length_factor = (title.len() as f32 / 50.0).clamp(0.5, 2.0);
            (base_duration as f32 * length_factor) as u64
        };

    Duration::from_secs(duration_seconds)
}

/// Estimate difficulty score from title content
fn estimate_difficulty_score_from_title(title: &str) -> f32 {
    let title_lower = title.to_lowercase();
    let mut score = 0.5f32; // Default intermediate

    let beginner_keywords = [
        "intro",
        "basic",
        "fundamentals",
        "getting started",
        "beginner",
        "overview",
    ];
    let advanced_keywords = [
        "advanced",
        "expert",
        "master",
        "deep",
        "optimization",
        "architecture",
        "complex",
    ];

    for keyword in &beginner_keywords {
        if title_lower.contains(keyword) {
            score -= 0.15;
        }
    }

    for keyword in &advanced_keywords {
        if title_lower.contains(keyword) {
            score += 0.15;
        }
    }

    score.clamp(0.0, 1.0)
}

/// Extract basic topic tags from title
fn extract_basic_topic_tags(title: &str) -> Vec<String> {
    let normalized = normalize_text(title);
    normalized
        .split_whitespace()
        .filter(|word| word.len() > 3)
        .take(3)
        .map(|s| s.to_string())
        .collect()
}

/// Extract topic tags from feature vector
fn extract_topic_tags_from_features(
    feature_vector: &crate::nlp::clustering::FeatureVector,
) -> Vec<String> {
    feature_vector
        .top_features(3)
        .into_iter()
        .map(|(term, _)| term)
        .collect()
}

/// Apply hierarchical clustering with automatic threshold determination
fn apply_hierarchical_clustering(
    titles: &[String],
) -> Result<(Vec<Module>, ClusteringMetadata), NlpError> {
    let start_time = Instant::now();

    // Step 1: Configure hierarchical clusterer
    let mut hierarchical_clusterer = HierarchicalClusterer::default();

    // Determine optimal threshold based on content characteristics
    let analysis = hierarchical_clusterer
        .analyze_content(titles)
        .map_err(|e| NlpError::Processing(format!("Hierarchical analysis failed: {e}")))?;

    let optimal_threshold =
        hierarchical_clusterer.determine_optimal_threshold(&analysis.feature_vectors);
    hierarchical_clusterer.distance_threshold = optimal_threshold;

    // Step 2: Perform hierarchical clustering
    let clusters = hierarchical_clusterer
        .cluster_videos(&analysis, 0)
        .map_err(|e| NlpError::Processing(format!("Hierarchical clustering failed: {e}")))?;

    // Step 3: Create video metadata for optimization
    let video_metadata = create_enhanced_video_metadata(titles, &analysis);

    // Step 4: Optimize clusters
    let optimized_clusters = hierarchical_clusterer
        .optimize_clusters(
            clusters.clone(),
            &video_metadata
                .iter()
                .map(|v| v.duration)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| NlpError::Processing(format!("Hierarchical optimization failed: {e}")))?;

    // Step 5: Apply duration balancing
    let default_settings = create_default_plan_settings();
    let target_duration =
        Duration::from_secs((default_settings.session_length_minutes * 60) as u64);
    let max_duration =
        Duration::from_secs((default_settings.session_length_minutes * 60 * 120 / 100) as u64); // 120% of target
    let duration_balancer = DurationBalancer::new(
        target_duration,
        max_duration,
        0.8, // Allow 80% utilization
    );

    let balanced_clusters = duration_balancer
        .balance_clusters(optimized_clusters)
        .map_err(|e| NlpError::Processing(format!("Hierarchical balancing failed: {e}")))?;

    // Step 6: Extract topics
    let topic_extractor = TopicExtractor::new(3, 0.15);
    let clustering_topics = topic_extractor.extract_topics(
        titles,
        &analysis
            .topic_keywords
            .iter()
            .enumerate()
            .map(|(i, keyword)| (keyword.clone(), 1.0 - (i as f32 * 0.1)))
            .collect(),
    );

    let extracted_topics: Vec<crate::types::TopicInfo> = clustering_topics
        .into_iter()
        .map(|topic| crate::types::TopicInfo {
            keyword: topic.keyword,
            relevance_score: topic.relevance_score,
            video_count: topic.related_videos.len(),
        })
        .collect();

    // Step 7: Convert to modules
    let modules = convert_balanced_clusters_to_modules_with_topics(
        balanced_clusters.clone(),
        &extracted_topics,
        titles,
    )?;

    // Step 8: Generate metadata
    let sections: Vec<Section> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| Section {
            title: title.clone(),
            video_index: i,
            duration: estimate_video_duration(title).unwrap_or_else(|| Duration::from_secs(600)),
        })
        .collect();

    let input_metrics =
        crate::nlp::clustering::metadata_generator::InputMetricsCalculator::calculate_metrics(
            &sections,
        );
    let performance_metrics = crate::nlp::clustering::PerformanceMetrics {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        content_analysis_time_ms: (start_time.elapsed().as_millis() / 3) as u64,
        clustering_time_ms: (start_time.elapsed().as_millis() / 3) as u64,
        optimization_time_ms: (start_time.elapsed().as_millis() / 3) as u64,
        peak_memory_usage_bytes: 1024 * 1024, // 1MB estimate
        algorithm_iterations: clusters.len() as u32,
        input_metrics,
    };

    let clustering_metadata = crate::nlp::clustering::metadata_generator::MetadataGenerator::generate_complete_metadata_from_balanced(
        &sections,
        &balanced_clusters,
        crate::types::ClusteringAlgorithm::Hierarchical,
        crate::types::ClusteringStrategy::ContentBased,
        optimal_threshold,
        extracted_topics,
        performance_metrics,
    );

    log::info!(
        "Hierarchical clustering completed: {} modules, threshold: {:.3}, time: {}ms",
        modules.len(),
        optimal_threshold,
        clustering_metadata.processing_time_ms
    );

    Ok((modules, clustering_metadata))
}

/// Apply LDA topic modeling for content clustering
fn apply_lda_clustering(titles: &[String]) -> Result<(Vec<Module>, ClusteringMetadata), NlpError> {
    let start_time = Instant::now();

    // Step 1: Configure LDA clusterer
    let mut lda_clusterer = LdaClusterer::default();
    let optimal_topics = lda_clusterer.determine_optimal_topics(titles);
    lda_clusterer.num_topics = optimal_topics;

    // Step 2: Fit LDA model
    let lda_model = lda_clusterer
        .fit_lda(titles)
        .map_err(|e| NlpError::Processing(format!("LDA fitting failed: {e}")))?;

    // Step 3: Cluster videos based on topics
    let clusters = lda_clusterer
        .cluster_by_topics(&lda_model, 0.3)
        .map_err(|e| NlpError::Processing(format!("LDA clustering failed: {e}")))?;

    // Step 4: Create video metadata for optimization
    let video_metadata: Vec<VideoWithMetadata> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| VideoWithMetadata {
            index: i,
            title: title.clone(),
            duration: estimate_enhanced_video_duration(title),
            feature_vector: crate::nlp::clustering::FeatureVector::default(),
            difficulty_score: estimate_difficulty_score_from_title(title),
            topic_tags: extract_basic_topic_tags(title),
        })
        .collect();

    // Step 5: Optimize clusters
    let optimized_clusters = lda_clusterer
        .optimize_clusters(
            clusters.clone(),
            &video_metadata
                .iter()
                .map(|v| v.duration)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| NlpError::Processing(format!("LDA optimization failed: {e}")))?;

    // Step 6: Apply duration balancing
    let default_settings = create_default_plan_settings();
    let target_duration =
        Duration::from_secs((default_settings.session_length_minutes * 60) as u64);
    let max_duration =
        Duration::from_secs((default_settings.session_length_minutes * 60 * 120 / 100) as u64); // 120% of target
    let duration_balancer = DurationBalancer::new(target_duration, max_duration, 0.8);

    let balanced_clusters = duration_balancer
        .balance_clusters(optimized_clusters)
        .map_err(|e| NlpError::Processing(format!("LDA balancing failed: {e}")))?;

    // Step 7: Extract topics from LDA model
    let extracted_topics: Vec<crate::types::TopicInfo> = lda_model
        .topics
        .into_iter()
        .map(|topic| crate::types::TopicInfo {
            keyword: topic
                .top_words
                .first()
                .map(|(word, _)| word.clone())
                .unwrap_or_default(),
            relevance_score: topic
                .top_words
                .first()
                .map(|(_, score)| *score)
                .unwrap_or(0.0),
            video_count: 0, // Will be calculated later
        })
        .collect();

    // Step 8: Convert to modules
    let modules = convert_balanced_clusters_to_modules_with_topics(
        balanced_clusters.clone(),
        &extracted_topics,
        titles,
    )?;

    // Step 9: Generate metadata
    let sections: Vec<Section> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| Section {
            title: title.clone(),
            video_index: i,
            duration: estimate_video_duration(title).unwrap_or_else(|| Duration::from_secs(600)),
        })
        .collect();

    let input_metrics =
        crate::nlp::clustering::metadata_generator::InputMetricsCalculator::calculate_metrics(
            &sections,
        );
    let performance_metrics = crate::nlp::clustering::PerformanceMetrics {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        content_analysis_time_ms: (start_time.elapsed().as_millis() / 4) as u64,
        clustering_time_ms: (start_time.elapsed().as_millis() / 2) as u64, // LDA takes more time
        optimization_time_ms: (start_time.elapsed().as_millis() / 4) as u64,
        peak_memory_usage_bytes: 1536 * 1024, // 1.5MB estimate for LDA
        algorithm_iterations: lda_clusterer.num_iterations as u32,
        input_metrics,
    };

    let clustering_metadata = crate::nlp::clustering::metadata_generator::MetadataGenerator::generate_complete_metadata_from_balanced(
        &sections,
        &balanced_clusters,
        crate::types::ClusteringAlgorithm::Lda,
        crate::types::ClusteringStrategy::ContentBased,
        0.3, // LDA similarity threshold
        extracted_topics,
        performance_metrics,
    );

    log::info!(
        "LDA clustering completed: {} modules, {} topics, time: {}ms",
        modules.len(),
        optimal_topics,
        clustering_metadata.processing_time_ms
    );

    Ok((modules, clustering_metadata))
}

/// Apply advanced hybrid clustering using multiple algorithms
fn apply_advanced_hybrid_clustering_v2(
    titles: &[String],
) -> Result<(Vec<Module>, ClusteringMetadata), NlpError> {
    let start_time = Instant::now();

    // Step 1: Configure hybrid clusterer with automatic strategy selection
    let hybrid_clusterer = HybridClusterer::new(
        StrategySelection::Automatic,
        EnsembleMethod::BestQuality,
        0.6,
    );

    // Step 2: Perform hybrid clustering
    let ensemble_results = hybrid_clusterer
        .cluster_hybrid(titles)
        .map_err(|e| NlpError::Processing(format!("Hybrid clustering failed: {e}")))?;

    // Step 3: Create video metadata for optimization
    let video_metadata: Vec<VideoWithMetadata> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| VideoWithMetadata {
            index: i,
            title: title.clone(),
            duration: estimate_enhanced_video_duration(title),
            feature_vector: crate::nlp::clustering::FeatureVector::default(),
            difficulty_score: estimate_difficulty_score_from_title(title),
            topic_tags: extract_basic_topic_tags(title),
        })
        .collect();

    // Step 4: Optimize the best clusters
    let optimized_clusters = hybrid_clusterer
        .optimize_clusters(
            ensemble_results.final_clusters.clone(),
            &video_metadata
                .iter()
                .map(|v| v.duration)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| NlpError::Processing(format!("Hybrid optimization failed: {e}")))?;

    // Step 5: Apply duration balancing
    let default_settings = create_default_plan_settings();
    let target_duration =
        Duration::from_secs((default_settings.session_length_minutes * 60) as u64);
    let max_duration =
        Duration::from_secs((default_settings.session_length_minutes * 60 * 120 / 100) as u64); // 120% of target
    let duration_balancer = DurationBalancer::new(target_duration, max_duration, 0.8);

    let balanced_clusters = duration_balancer
        .balance_clusters(optimized_clusters)
        .map_err(|e| NlpError::Processing(format!("Hybrid balancing failed: {e}")))?;

    // Step 6: Extract topics from the best performing algorithm
    let topic_extractor = TopicExtractor::new(3, 0.12);
    let clustering_topics = topic_extractor.extract_topics(
        titles,
        &std::collections::HashMap::new(), // Use default topic extraction
    );

    let extracted_topics: Vec<crate::types::TopicInfo> = clustering_topics
        .into_iter()
        .map(|topic| crate::types::TopicInfo {
            keyword: topic.keyword,
            relevance_score: topic.relevance_score,
            video_count: topic.related_videos.len(),
        })
        .collect();

    // Step 7: Convert to modules
    let modules = convert_balanced_clusters_to_modules_with_topics(
        balanced_clusters.clone(),
        &extracted_topics,
        titles,
    )?;

    // Step 8: Generate metadata with ensemble information
    let sections: Vec<Section> = titles
        .iter()
        .enumerate()
        .map(|(i, title)| Section {
            title: title.clone(),
            video_index: i,
            duration: estimate_video_duration(title).unwrap_or_else(|| Duration::from_secs(600)),
        })
        .collect();

    let input_metrics =
        crate::nlp::clustering::metadata_generator::InputMetricsCalculator::calculate_metrics(
            &sections,
        );
    let performance_metrics = crate::nlp::clustering::PerformanceMetrics {
        total_processing_time_ms: start_time.elapsed().as_millis() as u64,
        content_analysis_time_ms: (start_time.elapsed().as_millis() / 4) as u64,
        clustering_time_ms: (start_time.elapsed().as_millis() / 2) as u64, // Multiple algorithms
        optimization_time_ms: (start_time.elapsed().as_millis() / 4) as u64,
        peak_memory_usage_bytes: 2048 * 1024, // 2MB estimate for multiple algorithms
        algorithm_iterations: ensemble_results.quality_scores.len() as u32,
        input_metrics,
    };

    let clustering_metadata = crate::nlp::clustering::metadata_generator::MetadataGenerator::generate_complete_metadata_from_balanced(
        &sections,
        &balanced_clusters,
        crate::types::ClusteringAlgorithm::Hybrid,
        crate::types::ClusteringStrategy::Hybrid,
        0.6, // Hybrid threshold
        extracted_topics,
        performance_metrics,
    );

    log::info!(
        "Advanced hybrid clustering completed: {} modules, selected: {}, quality scores: {:?}, time: {}ms",
        modules.len(),
        ensemble_results.selected_algorithm,
        ensemble_results.quality_scores,
        clustering_metadata.processing_time_ms
    );

    Ok((modules, clustering_metadata))
}

/// Convert balanced clusters to modules with topic information
fn convert_balanced_clusters_to_modules_with_topics(
    balanced_clusters: Vec<BalancedCluster>,
    topics: &[crate::types::TopicInfo],
    titles: &[String],
) -> Result<Vec<Module>, NlpError> {
    let mut modules = Vec::new();

    for (i, cluster) in balanced_clusters.into_iter().enumerate() {
        let sections: Vec<Section> = cluster
            .videos
            .into_iter()
            .map(|video| Section {
                title: titles.get(video.index).cloned().unwrap_or_default(),
                video_index: video.index,
                duration: video.duration,
            })
            .collect();

        let total_duration = sections.iter().map(|s| s.duration).sum();

        // Generate module title from topics or use default
        let module_title = if let Some(topic) = topics.get(i % topics.len()) {
            format!("Module {}: {}", i + 1, topic.keyword)
        } else {
            format!("Module {}", i + 1)
        };

        modules.push(Module {
            title: module_title,
            sections,
            total_duration,
            similarity_score: Some(0.5), // Default similarity score
            topic_keywords: Vec::new(),
            difficulty_level: Some(DifficultyLevel::Intermediate),
        });
    }

    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structure_course_basic() {
        let titles = vec![
            "Introduction to Programming".to_string(),
            "Variables and Data Types".to_string(),
            "Control Structures".to_string(),
            "Functions".to_string(),
        ];

        let result = structure_course(titles).unwrap();
        assert!(!result.modules.is_empty());
        assert_eq!(result.metadata.total_videos, 4);
    }

    #[test]
    fn test_structure_course_with_modules() {
        let titles = vec![
            "Module 1: Introduction".to_string(),
            "Lesson 1: Getting Started".to_string(),
            "Lesson 2: Basic Concepts".to_string(),
            "Module 2: Advanced Topics".to_string(),
            "Lesson 3: Complex Examples".to_string(),
        ];

        let result = structure_course(titles).unwrap();
        println!("Result modules: {}", result.modules.len());
        for (i, module) in result.modules.iter().enumerate() {
            println!(
                "Module {}: {} with {} sections",
                i,
                module.title,
                module.sections.len()
            );
        }
        println!("Is clustered: {}", result.is_clustered());
        assert_eq!(result.modules.len(), 2);
    }

    #[test]
    fn test_empty_titles() {
        let result = structure_course(vec![]);
        assert!(matches!(result, Err(NlpError::InvalidInput(_))));
    }

    #[test]
    fn test_difficulty_estimation() {
        let beginner_titles = vec!["Introduction to Basics".to_string()];
        let advanced_titles = vec!["Advanced Optimization Techniques".to_string()];

        assert!(matches!(
            estimate_difficulty(&beginner_titles),
            DifficultyLevel::Beginner
        ));
        assert!(matches!(
            estimate_difficulty(&advanced_titles),
            DifficultyLevel::Advanced
        ));
    }

    #[test]
    fn test_chunk_size_calculation() {
        assert_eq!(calculate_optimal_chunk_size(10), 3);
        assert_eq!(calculate_optimal_chunk_size(30), 6);
        assert_eq!(calculate_optimal_chunk_size(100), 14);
    }
}
// ============================================================================
// ADVANCED CLUSTERING CONVERSION AND QUALITY FUNCTIONS
// ============================================================================

/// Convert balanced clusters to modules with intelligent naming
fn convert_balanced_clusters_to_modules(
    balanced_clusters: Vec<BalancedCluster>,
    topics: &[TopicInfo],
    topic_extractor: &TopicExtractor,
    titles: &[String],
) -> Result<Vec<Module>, NlpError> {
    let mut modules = Vec::new();

    for (i, cluster) in balanced_clusters.into_iter().enumerate() {
        let sections: Vec<Section> = cluster
            .videos
            .into_iter()
            .map(|video| Section {
                title: video.title,
                video_index: video.index,
                duration: video.duration,
            })
            .collect();

        // Generate intelligent module title
        let module_title =
            generate_intelligent_module_title(&sections, topics, topic_extractor, i + 1);

        // Estimate difficulty from video metadata
        let difficulty_level = estimate_module_difficulty(&sections, titles);

        // Calculate similarity score from cluster balance
        let similarity_score = cluster.balance_score;

        // Extract topic keywords for this cluster
        let topic_keywords = extract_cluster_topic_keywords(&sections, topics);

        let module = Module::new_with_clustering(
            module_title,
            sections,
            similarity_score,
            topic_keywords,
            difficulty_level,
        );

        modules.push(module);
    }

    Ok(modules)
}

/// Convert balanced clusters to modules with duration focus
fn convert_balanced_clusters_to_modules_duration_focused(
    balanced_clusters: Vec<BalancedCluster>,
    topics: &[TopicInfo],
    titles: &[String],
) -> Result<Vec<Module>, NlpError> {
    let mut modules = Vec::new();

    for (i, cluster) in balanced_clusters.into_iter().enumerate() {
        let sections: Vec<Section> = cluster
            .videos
            .into_iter()
            .map(|video| Section {
                title: video.title,
                video_index: video.index,
                duration: video.duration,
            })
            .collect();

        // Generate duration-focused module title
        let module_title = generate_duration_focused_module_title(
            &sections,
            topics,
            cluster.total_duration,
            i + 1,
        );

        let difficulty_level = estimate_module_difficulty(&sections, titles);
        let similarity_score = cluster.balance_score * 0.8; // Slightly lower for duration-focused
        let topic_keywords = extract_cluster_topic_keywords(&sections, topics);

        let module = Module::new_with_clustering(
            module_title,
            sections,
            similarity_score,
            topic_keywords,
            difficulty_level,
        );

        modules.push(module);
    }

    Ok(modules)
}

/// Convert balanced clusters to modules with hybrid approach
#[allow(dead_code)]
fn convert_balanced_clusters_to_modules_hybrid(
    balanced_clusters: Vec<BalancedCluster>,
    topics: &[TopicInfo],
    topic_extractor: &TopicExtractor,
    titles: &[String],
    analysis: &AdvancedContentAnalysis,
) -> Result<Vec<Module>, NlpError> {
    let mut modules = Vec::new();

    for (i, cluster) in balanced_clusters.into_iter().enumerate() {
        let sections: Vec<Section> = cluster
            .videos
            .into_iter()
            .map(|video| Section {
                title: video.title,
                video_index: video.index,
                duration: video.duration,
            })
            .collect();

        // Generate hybrid module title considering both content and duration
        let module_title = generate_hybrid_module_title(
            &sections,
            topics,
            topic_extractor,
            cluster.total_duration,
            analysis,
            i + 1,
        );

        let difficulty_level = estimate_module_difficulty(&sections, titles);
        let similarity_score = cluster.balance_score;
        let topic_keywords = extract_cluster_topic_keywords(&sections, topics);

        let module = Module::new_with_clustering(
            module_title,
            sections,
            similarity_score,
            topic_keywords,
            difficulty_level,
        );

        modules.push(module);
    }

    Ok(modules)
}

/// Generate intelligent module title using topic analysis
fn generate_intelligent_module_title(
    sections: &[Section],
    topics: &[TopicInfo],
    topic_extractor: &TopicExtractor,
    module_number: usize,
) -> String {
    // Extract keywords from section titles
    let section_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let cluster_keywords = extract_common_keywords(&section_titles);

    // Find best matching topic
    let best_topic = find_best_matching_topic(&cluster_keywords, topics);

    if let Some(topic) = best_topic {
        if topic.relevance_score > 0.6 {
            return capitalize_first(&topic.keyword).to_string();
        }
    }

    // Fallback to generated title from topic extractor
    if !cluster_keywords.is_empty() {
        let generated_title = topic_extractor.generate_cluster_title(&cluster_keywords);
        if !generated_title.is_empty() && generated_title != "Cluster" {
            return generated_title;
        }
    }

    // Final fallback
    format!("Module {module_number}")
}

/// Generate duration-focused module title
fn generate_duration_focused_module_title(
    sections: &[Section],
    topics: &[TopicInfo],
    total_duration: Duration,
    module_number: usize,
) -> String {
    let duration_minutes = total_duration.as_secs() / 60;
    let section_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let keywords = extract_common_keywords(&section_titles);

    // Try to find a topic-based name first
    if let Some(topic) = find_best_matching_topic(&keywords, topics) {
        if topic.relevance_score > 0.4 {
            // Lower threshold for duration-focused
            return format!(
                "{} ({}m)",
                capitalize_first(&topic.keyword),
                duration_minutes
            );
        }
    }

    // Use first significant keyword if available
    if let Some(keyword) = keywords.first() {
        if keyword.len() > 3 {
            return format!(
                "{} Session ({}m)",
                capitalize_first(keyword),
                duration_minutes
            );
        }
    }

    format!("Session {module_number} ({duration_minutes}m)")
}

/// Generate hybrid module title balancing content and duration
#[allow(dead_code)]
fn generate_hybrid_module_title(
    sections: &[Section],
    topics: &[TopicInfo],
    topic_extractor: &TopicExtractor,
    total_duration: Duration,
    analysis: &AdvancedContentAnalysis,
    module_number: usize,
) -> String {
    let section_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let keywords = extract_common_keywords(&section_titles);

    // Decide whether to emphasize content or duration based on analysis
    let emphasize_content = analysis.content_diversity_score > analysis.duration_variance;

    if emphasize_content {
        // Content-focused naming with duration hint
        if let Some(topic) = find_best_matching_topic(&keywords, topics) {
            if topic.relevance_score > 0.5 {
                return capitalize_first(&topic.keyword).to_string();
            }
        }

        let generated_title = topic_extractor.generate_cluster_title(&keywords);
        if !generated_title.is_empty() && generated_title != "Cluster" {
            return generated_title;
        }
    } else {
        // Duration-focused naming with content hint
        let duration_minutes = total_duration.as_secs() / 60;
        if let Some(topic) = find_best_matching_topic(&keywords, topics) {
            if topic.relevance_score > 0.4 {
                return format!(
                    "{} ({}m)",
                    capitalize_first(&topic.keyword),
                    duration_minutes
                );
            }
        }
    }

    format!("Module {module_number}")
}

/// Find best matching topic for given keywords
fn find_best_matching_topic<'a>(
    keywords: &[String],
    topics: &'a [TopicInfo],
) -> Option<&'a TopicInfo> {
    let mut best_topic = None;
    let mut best_score = 0.0;

    for topic in topics {
        for keyword in keywords {
            if topic.keyword.contains(keyword) || keyword.contains(&topic.keyword) {
                let score =
                    topic.relevance_score * (keyword.len() as f32 / topic.keyword.len() as f32);
                if score > best_score {
                    best_score = score;
                    best_topic = Some(topic);
                }
            }
        }
    }

    best_topic
}

/// Extract topic keywords for a cluster
fn extract_cluster_topic_keywords(sections: &[Section], topics: &[TopicInfo]) -> Vec<String> {
    let section_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    let keywords = extract_common_keywords(&section_titles);

    let mut cluster_topics = Vec::new();

    // Add matching topics
    for topic in topics {
        for keyword in &keywords {
            if topic.keyword.contains(keyword) || keyword.contains(&topic.keyword) {
                cluster_topics.push(topic.keyword.clone());
                break;
            }
        }
    }

    // Add significant keywords not in topics
    for keyword in keywords.into_iter().take(3) {
        if !cluster_topics.contains(&keyword) {
            cluster_topics.push(keyword);
        }
    }

    cluster_topics
}

/// Estimate module difficulty from sections
fn estimate_module_difficulty(sections: &[Section], titles: &[String]) -> DifficultyLevel {
    let section_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();
    estimate_advanced_difficulty(
        &section_titles,
        &create_basic_analysis_for_difficulty(titles),
    )
}

/// Create basic analysis for difficulty estimation
fn create_basic_analysis_for_difficulty(titles: &[String]) -> AdvancedContentAnalysis {
    // This is a simplified version for difficulty estimation
    AdvancedContentAnalysis {
        content_analysis: ContentAnalysis {
            feature_vectors: Vec::new(),
            similarity_matrix: crate::nlp::clustering::SimilarityMatrix::new(0),
            vocabulary: std::collections::HashSet::new(),
            document_frequencies: HashMap::new(),
            topic_keywords: Vec::new(),
        },
        extracted_topics: Vec::new(),
        content_diversity_score: 0.5,
        title_similarity_score: 0.5,
        has_clear_topics: false,
        estimated_optimal_clusters: titles.len() / 5,
        content_complexity: 0.5,
        duration_variance: 0.3,
        clustering_feasibility: 0.5,
        vocabulary_richness: 0.5,
        topic_coherence_score: 0.5,
    }
}

/// Estimate advanced difficulty using content analysis
fn estimate_advanced_difficulty(
    titles: &[String],
    analysis: &AdvancedContentAnalysis,
) -> DifficultyLevel {
    let basic_difficulty = estimate_difficulty(titles);

    // Adjust based on content complexity
    match basic_difficulty {
        DifficultyLevel::Beginner => {
            if analysis.content_complexity > 0.7 {
                DifficultyLevel::Intermediate
            } else {
                DifficultyLevel::Beginner
            }
        }
        DifficultyLevel::Intermediate => {
            if analysis.content_complexity > 0.8 {
                DifficultyLevel::Advanced
            } else if analysis.content_complexity < 0.3 {
                DifficultyLevel::Beginner
            } else {
                DifficultyLevel::Intermediate
            }
        }
        DifficultyLevel::Advanced => {
            if analysis.content_complexity > 0.9 {
                DifficultyLevel::Expert
            } else {
                DifficultyLevel::Advanced
            }
        }
        DifficultyLevel::Expert => DifficultyLevel::Expert,
    }
}

/// Calculate comprehensive quality metrics
fn calculate_comprehensive_quality_metrics(
    video_clusters: &[VideoCluster],
    modules: &[Module],
    _content_analysis: &ContentAnalysis,
) -> QualityMetrics {
    let clustering_quality = calculate_clustering_quality_score(video_clusters);
    let content_coherence = calculate_advanced_coherence_score(modules);
    let duration_balance = calculate_duration_quality_score(modules);
    let topic_quality = calculate_topic_quality_score(modules);

    let overall_quality = (clustering_quality * 0.3
        + content_coherence * 0.3
        + duration_balance * 0.2
        + topic_quality * 0.2)
        .clamp(0.0, 1.0);

    QualityMetrics {
        overall_quality,
        clustering_quality,
        content_quality: content_coherence,
        duration_quality: duration_balance,
        topic_quality,
    }
}

/// Calculate hybrid quality metrics
#[allow(dead_code)]
fn calculate_hybrid_quality_metrics(
    video_clusters: &[VideoCluster],
    modules: &[Module],
    _content_analysis: &ContentAnalysis,
    advanced_analysis: &AdvancedContentAnalysis,
) -> QualityMetrics {
    let clustering_quality = calculate_clustering_quality_score(video_clusters);
    let content_coherence = calculate_advanced_coherence_score(modules);
    let duration_balance = calculate_duration_quality_score(modules);
    let topic_quality = calculate_topic_quality_score(modules);
    let feasibility_bonus = advanced_analysis.clustering_feasibility * 0.1;

    let overall_quality = (clustering_quality * 0.25
        + content_coherence * 0.25
        + duration_balance * 0.25
        + topic_quality * 0.15
        + feasibility_bonus)
        .clamp(0.0, 1.0);

    QualityMetrics {
        overall_quality,
        clustering_quality,
        content_quality: content_coherence,
        duration_quality: duration_balance,
        topic_quality,
    }
}

/// Calculate duration quality score for modules
fn calculate_duration_quality_score(modules: &[Module]) -> f32 {
    if modules.is_empty() {
        return 0.0;
    }

    let durations: Vec<f32> = modules
        .iter()
        .map(|m| m.total_duration.as_secs() as f32)
        .collect();
    let mean = durations.iter().sum::<f32>() / durations.len() as f32;
    let variance =
        durations.iter().map(|&d| (d - mean).powi(2)).sum::<f32>() / durations.len() as f32;
    let coefficient_of_variation = variance.sqrt() / mean;

    // Lower coefficient of variation means better balance
    (1.0 - coefficient_of_variation.clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

/// Calculate topic quality score for modules
fn calculate_topic_quality_score(modules: &[Module]) -> f32 {
    if modules.is_empty() {
        return 0.0;
    }

    let modules_with_topics = modules
        .iter()
        .filter(|m| !m.topic_keywords.is_empty())
        .count();
    let topic_coverage = modules_with_topics as f32 / modules.len() as f32;

    let avg_topic_count = modules
        .iter()
        .map(|m| m.topic_keywords.len() as f32)
        .sum::<f32>()
        / modules.len() as f32;

    let topic_richness = (avg_topic_count / 3.0).clamp(0.0, 1.0); // Normalize around 3 topics per module

    (topic_coverage * 0.6 + topic_richness * 0.4).clamp(0.0, 1.0)
}

/// Calculate advanced coherence score for modules
fn calculate_advanced_coherence_score(modules: &[Module]) -> f32 {
    if modules.is_empty() {
        return 0.0;
    }

    let similarity_scores: Vec<f32> = modules.iter().filter_map(|m| m.similarity_score).collect();

    if similarity_scores.is_empty() {
        return 0.5; // Default score when no similarity data available
    }

    let avg_similarity = similarity_scores.iter().sum::<f32>() / similarity_scores.len() as f32;
    avg_similarity.clamp(0.0, 1.0)
}

// ============================================================================
// HELPER FUNCTIONS FOR DURATION-BASED CLUSTERING
// ============================================================================

/// Create duration-based initial clusters
fn create_duration_based_initial_clusters(
    video_metadata: &[VideoWithMetadata],
    settings: &PlanSettings,
) -> Result<Vec<OptimizedCluster>, NlpError> {
    let target_duration = Duration::from_secs(settings.session_length_minutes as u64 * 60);
    let mut clusters = Vec::new();
    let mut current_cluster_videos = Vec::new();
    let mut current_duration = Duration::from_secs(0);

    for video in video_metadata {
        if current_duration + video.duration > target_duration && !current_cluster_videos.is_empty()
        {
            // Create cluster from current videos
            let cluster = create_optimized_cluster_from_videos(
                std::mem::take(&mut current_cluster_videos),
                current_duration,
            );
            clusters.push(cluster);
            current_duration = Duration::from_secs(0);
        }

        current_duration += video.duration;
        current_cluster_videos.push(video.clone());
    }

    // Handle remaining videos
    if !current_cluster_videos.is_empty() {
        let cluster =
            create_optimized_cluster_from_videos(current_cluster_videos, current_duration);
        clusters.push(cluster);
    }

    Ok(clusters)
}

/// Create optimized cluster from videos
fn create_optimized_cluster_from_videos(
    videos: Vec<VideoWithMetadata>,
    total_duration: Duration,
) -> OptimizedCluster {
    let avg_difficulty =
        videos.iter().map(|v| v.difficulty_score).sum::<f32>() / videos.len() as f32;
    let difficulty_level = match avg_difficulty {
        s if s < 0.3 => DifficultyLevel::Beginner,
        s if s < 0.6 => DifficultyLevel::Intermediate,
        s if s < 0.8 => DifficultyLevel::Advanced,
        _ => DifficultyLevel::Expert,
    };

    let suggested_title = if let Some(first_video) = videos.first() {
        generate_cluster_title_from_video(&first_video.title)
    } else {
        "Untitled Cluster".to_string()
    };

    OptimizedCluster {
        videos,
        total_duration,
        average_similarity: 0.6, // Default for duration-based
        difficulty_level,
        suggested_title,
    }
}

/// Generate cluster title from video title
fn generate_cluster_title_from_video(video_title: &str) -> String {
    let words: Vec<&str> = video_title.split_whitespace().take(3).collect();
    if words.len() >= 2 {
        format!("{} {}", capitalize_first(words[0]), words[1])
    } else if !words.is_empty() {
        capitalize_first(words[0])
    } else {
        "Content".to_string()
    }
}

/// Calculate basic TF-IDF scores for duration-based clustering
fn calculate_basic_tfidf_scores(titles: &[String]) -> HashMap<String, f32> {
    let mut word_counts = HashMap::new();
    let mut doc_counts = HashMap::new();

    // Count word frequencies
    for title in titles {
        let words: std::collections::HashSet<String> = normalize_text(title)
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .map(|w| w.to_string())
            .collect();

        for word in &words {
            *word_counts.entry(word.clone()).or_insert(0) += 1;
            *doc_counts.entry(word.clone()).or_insert(0) += 1;
        }
    }

    // Calculate simple TF-IDF scores
    let total_docs = titles.len() as f32;
    let mut tfidf_scores = HashMap::new();

    for (word, tf) in word_counts {
        let df = doc_counts.get(&word).unwrap_or(&1);
        let idf = (total_docs / *df as f32).ln();
        let tfidf = tf as f32 * idf;
        tfidf_scores.insert(word, tfidf);
    }

    tfidf_scores
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Estimate vocabulary size from titles
fn estimate_vocabulary_size(titles: &[String]) -> usize {
    let mut words = std::collections::HashSet::new();
    for title in titles {
        for word in normalize_text(title).split_whitespace() {
            if word.len() > 2 {
                words.insert(word.to_string());
            }
        }
    }
    words.len()
}

/// Estimate content complexity from titles
fn estimate_content_complexity_from_titles(titles: &[String]) -> f32 {
    let vocab_size = estimate_vocabulary_size(titles);
    let avg_title_length =
        titles.iter().map(|t| t.len()).sum::<usize>() as f32 / titles.len() as f32;
    let vocab_ratio = vocab_size as f32 / titles.len() as f32;
    let length_factor = (avg_title_length / 50.0).clamp(0.5, 2.0);

    (vocab_ratio * length_factor).clamp(0.0, 1.0)
}

/// Quality metrics structure
#[derive(Debug)]
struct QualityMetrics {
    overall_quality: f32,
    // TODO: Integrate later - Could be used for clustering algorithm selection
    #[allow(dead_code)]
    clustering_quality: f32,
    #[allow(dead_code)]
    content_quality: f32,
    #[allow(dead_code)]
    duration_quality: f32,
    // TODO: Integrate later - Could be used for topic coherence assessment
    #[allow(dead_code)]
    topic_quality: f32,
}
