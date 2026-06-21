//! Integration tests for Course Pilot ingestion pipelines.
//!
//! These tests use in-memory repositories to test the full ingestion
//! pipeline end-to-end without external dependencies.

use std::sync::Arc;
use std::sync::Mutex;

use course_pilot::domain::{
    entities::{Course, Exam, Module, Video},
    ports::{
        CourseRepository, ExamRepository, ExaminerAI, FetchError, LLMError, LocalMediaError,
        LocalMediaScanner, MCQuestion, ModuleRepository, PlaylistFetcher, RawLocalMediaMetadata,
        RepositoryError, SearchEntry, SearchRepository, SummarizerAI, TranscriptError,
        TranscriptProvider, VideoRepository,
    },
    services::{BoundaryDetector, TranscriptChunker},
    value_objects::{
        CourseId, ExamDifficulty, ExamId, ModuleId, PlaylistUrl, VideoId, VideoSource,
        YouTubeVideoId,
    },
};

use course_pilot::application::use_cases::{
    GenerateExamInput, SubmitExamInput, SummarizeVideoInput, SummarizeVideoOutput,
    SummarizeVideoUseCase, TakeExamUseCase,
};

// ─── Mock Scanner ───────────────────────────────────────────────────────

struct MockScanner {
    media: Vec<RawLocalMediaMetadata>,
}

impl MockScanner {
    fn new(media: Vec<RawLocalMediaMetadata>) -> Self {
        Self { media }
    }
}

#[async_trait::async_trait]
impl LocalMediaScanner for MockScanner {
    async fn scan(&self, _root: &str) -> Result<Vec<RawLocalMediaMetadata>, LocalMediaError> {
        Ok(self.media.clone())
    }
}

// ─── Mock Fetcher ───────────────────────────────────────────────────────

struct MockFetcher {
    videos: Vec<course_pilot::domain::ports::RawVideoMetadata>,
    should_fail: bool,
}

impl MockFetcher {
    fn new(videos: Vec<course_pilot::domain::ports::RawVideoMetadata>) -> Self {
        Self { videos, should_fail: false }
    }

    fn with_failure() -> Self {
        Self { videos: vec![], should_fail: true }
    }
}

#[async_trait::async_trait]
impl PlaylistFetcher for MockFetcher {
    async fn fetch_playlist(
        &self,
        _url: &PlaylistUrl,
    ) -> Result<Vec<course_pilot::domain::ports::RawVideoMetadata>, FetchError> {
        if self.should_fail {
            return Err(FetchError::NotFound("test_not_found".to_string()));
        }
        Ok(self.videos.clone())
    }
}

// ─── In-Memory Test Repositories ───────────────────────────────────────

struct InMemoryCourseRepo {
    courses: Mutex<Vec<Course>>,
}

impl InMemoryCourseRepo {
    fn new() -> Self {
        Self { courses: Mutex::new(vec![]) }
    }
}

impl CourseRepository for InMemoryCourseRepo {
    fn save(&self, course: &Course) -> Result<(), RepositoryError> {
        let mut c = self.courses.lock().unwrap();
        if let Some(pos) = c.iter().position(|e| e.id() == course.id()) {
            c[pos] = course.clone();
        } else {
            c.push(course.clone());
        }
        Ok(())
    }

    fn save_batch(&self, courses: &[Course]) -> Result<(), RepositoryError> {
        for course in courses {
            self.save(course)?;
        }
        Ok(())
    }

    fn find_by_id(&self, id: &CourseId) -> Result<Option<Course>, RepositoryError> {
        let c = self.courses.lock().unwrap();
        Ok(c.iter().find(|e| e.id() == id).cloned())
    }

    fn find_all(&self) -> Result<Vec<Course>, RepositoryError> {
        let c = self.courses.lock().unwrap();
        Ok(c.clone())
    }

    fn update_metadata(
        &self,
        id: &CourseId,
        _name: &str,
        _description: Option<&str>,
    ) -> Result<(), RepositoryError> {
        let _ = id;
        Ok(())
    }

    fn delete(&self, id: &CourseId) -> Result<(), RepositoryError> {
        let mut c = self.courses.lock().unwrap();
        c.retain(|e| e.id() != id);
        Ok(())
    }

    fn find_by_source_hash(&self, _hash: &str) -> Result<Option<Course>, RepositoryError> {
        Ok(None)
    }
}

struct InMemoryModuleRepo {
    modules: Mutex<Vec<Module>>,
}

impl InMemoryModuleRepo {
    fn new() -> Self {
        Self { modules: Mutex::new(vec![]) }
    }
}

impl ModuleRepository for InMemoryModuleRepo {
    fn save(&self, module: &Module) -> Result<(), RepositoryError> {
        let mut m = self.modules.lock().unwrap();
        if let Some(pos) = m.iter().position(|e| e.id() == module.id()) {
            m[pos] = module.clone();
        } else {
            m.push(module.clone());
        }
        Ok(())
    }

    fn save_batch(&self, modules: &[Module]) -> Result<(), RepositoryError> {
        for module in modules {
            self.save(module)?;
        }
        Ok(())
    }

    fn find_by_id(&self, id: &ModuleId) -> Result<Option<Module>, RepositoryError> {
        let m = self.modules.lock().unwrap();
        Ok(m.iter().find(|e| e.id() == id).cloned())
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Module>, RepositoryError> {
        let m = self.modules.lock().unwrap();
        let mut result: Vec<Module> =
            m.iter().filter(|e| e.course_id() == course_id).cloned().collect();
        result.sort_by_key(|e| e.sort_order());
        Ok(result)
    }

    fn update_title(&self, _id: &ModuleId, _title: &str) -> Result<(), RepositoryError> {
        Ok(())
    }

    fn delete(&self, id: &ModuleId) -> Result<(), RepositoryError> {
        let mut m = self.modules.lock().unwrap();
        m.retain(|e| e.id() != id);
        Ok(())
    }
}

struct InMemoryVideoRepo {
    videos: Mutex<Vec<Video>>,
    module_repo: Arc<InMemoryModuleRepo>,
}

impl InMemoryVideoRepo {
    fn new(module_repo: Arc<InMemoryModuleRepo>) -> Self {
        Self { videos: Mutex::new(vec![]), module_repo }
    }
}

impl VideoRepository for InMemoryVideoRepo {
    fn save(&self, video: &Video) -> Result<(), RepositoryError> {
        let mut v = self.videos.lock().unwrap();
        if let Some(pos) = v.iter().position(|e| e.id() == video.id()) {
            v[pos] = video.clone();
        } else {
            v.push(video.clone());
        }
        Ok(())
    }

    fn save_batch(&self, videos: &[Video]) -> Result<(), RepositoryError> {
        for video in videos {
            self.save(video)?;
        }
        Ok(())
    }

    fn find_by_id(&self, id: &VideoId) -> Result<Option<Video>, RepositoryError> {
        let v = self.videos.lock().unwrap();
        Ok(v.iter().find(|e| e.id() == id).cloned())
    }

    fn find_by_module(&self, module_id: &ModuleId) -> Result<Vec<Video>, RepositoryError> {
        let v = self.videos.lock().unwrap();
        let mut result: Vec<Video> =
            v.iter().filter(|e| e.module_id() == module_id).cloned().collect();
        result.sort_by_key(|e| e.sort_order());
        Ok(result)
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Video>, RepositoryError> {
        let modules = self.module_repo.find_by_course(course_id)?;
        let module_map: std::collections::HashMap<ModuleId, u32> =
            modules.into_iter().map(|m| (*m.id(), m.sort_order())).collect();

        let v = self.videos.lock().unwrap();
        let mut result: Vec<Video> =
            v.iter().filter(|video| module_map.contains_key(video.module_id())).cloned().collect();

        result.sort_by(|a, b| {
            let a_mod_order = module_map.get(a.module_id()).copied().unwrap_or(0);
            let b_mod_order = module_map.get(b.module_id()).copied().unwrap_or(0);
            match a_mod_order.cmp(&b_mod_order) {
                std::cmp::Ordering::Equal => a.sort_order().cmp(&b.sort_order()),
                other => other,
            }
        });

        Ok(result)
    }

    fn update_completion(&self, id: &VideoId, completed: bool) -> Result<(), RepositoryError> {
        let mut v = self.videos.lock().unwrap();
        if let Some(pos) = v.iter().position(|e| e.id() == id) {
            if completed {
                v[pos].mark_completed();
            } else {
                v[pos].mark_pending();
            }
        }
        Ok(())
    }

    fn update_transcript(
        &self,
        id: &VideoId,
        transcript: Option<&str>,
    ) -> Result<(), RepositoryError> {
        let mut v = self.videos.lock().unwrap();
        if let Some(pos) = v.iter().position(|e| e.id() == id) {
            v[pos].update_transcript(transcript.map(|s| s.to_string()));
        }
        Ok(())
    }

    fn update_summary(&self, id: &VideoId, summary: Option<&str>) -> Result<(), RepositoryError> {
        let mut v = self.videos.lock().unwrap();
        if let Some(pos) = v.iter().position(|e| e.id() == id) {
            v[pos].update_summary(summary.map(|s| s.to_string()));
        }
        Ok(())
    }

    fn update_module(
        &self,
        _id: &VideoId,
        _module_id: &ModuleId,
        _sort_order: u32,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }

    fn swap_video_orders(
        &self,
        video_a_id: &VideoId,
        video_b_id: &VideoId,
    ) -> Result<(), RepositoryError> {
        let mut v = self.videos.lock().unwrap();
        let a_idx = v
            .iter()
            .position(|e| e.id() == video_a_id)
            .ok_or_else(|| RepositoryError::Database(format!("Video not found: {}", video_a_id)))?;
        let b_idx = v
            .iter()
            .position(|e| e.id() == video_b_id)
            .ok_or_else(|| RepositoryError::Database(format!("Video not found: {}", video_b_id)))?;
        let a_order = v[a_idx].sort_order();
        let b_order = v[b_idx].sort_order();
        v[a_idx].set_sort_order(b_order);
        v[b_idx].set_sort_order(a_order);
        Ok(())
    }

    fn delete(&self, _id: &VideoId) -> Result<(), RepositoryError> {
        Ok(())
    }
}

struct InMemorySearchRepo;

impl SearchRepository for InMemorySearchRepo {
    fn search(
        &self,
        _query: &str,
        _limit: usize,
    ) -> Result<Vec<course_pilot::domain::entities::SearchResult>, RepositoryError> {
        Ok(vec![])
    }
    fn index_course(
        &self,
        _course_id: &CourseId,
        _name: &str,
        _description: Option<&str>,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }
    fn index_video(
        &self,
        _video_id: &str,
        _title: &str,
        _description: Option<&str>,
        _course_id: &CourseId,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }
    fn index_note(
        &self,
        _note_id: &str,
        _video_title: &str,
        _content: &str,
        _course_id: &CourseId,
    ) -> Result<(), RepositoryError> {
        Ok(())
    }
    fn index_batch(&self, _entries: &[SearchEntry]) -> Result<(), RepositoryError> {
        Ok(())
    }
    fn remove_from_index(&self, _entity_id: &str) -> Result<(), RepositoryError> {
        Ok(())
    }
}

// ─── Mock Transcript Provider & LLMs ───────────────────────────────────

struct InMemoryExamRepo {
    exams: Mutex<Vec<Exam>>,
}

impl InMemoryExamRepo {
    fn new() -> Self {
        Self { exams: Mutex::new(vec![]) }
    }
}

impl ExamRepository for InMemoryExamRepo {
    fn save(&self, exam: &Exam) -> Result<(), RepositoryError> {
        let mut e = self.exams.lock().unwrap();
        if let Some(pos) = e.iter().position(|r| r.id() == exam.id()) {
            e[pos] = exam.clone();
        } else {
            e.push(exam.clone());
        }
        Ok(())
    }

    fn find_by_id(&self, id: &ExamId) -> Result<Option<Exam>, RepositoryError> {
        let e = self.exams.lock().unwrap();
        Ok(e.iter().find(|r| r.id() == id).cloned())
    }

    fn find_all(&self) -> Result<Vec<Exam>, RepositoryError> {
        let e = self.exams.lock().unwrap();
        Ok(e.clone())
    }

    fn find_by_video(&self, video_id: &VideoId) -> Result<Vec<Exam>, RepositoryError> {
        let e = self.exams.lock().unwrap();
        Ok(e.iter().filter(|r| r.video_id() == video_id).cloned().collect())
    }

    fn update_result(
        &self,
        id: &ExamId,
        score: f32,
        _passed: bool,
        user_answers_json: Option<String>,
    ) -> Result<(), RepositoryError> {
        let mut e = self.exams.lock().unwrap();
        if let Some(pos) = e.iter().position(|r| r.id() == id) {
            e[pos].record_result(score, user_answers_json);
        } else {
            return Err(RepositoryError::NotFound { entity: "Exam", id: id.to_string() });
        }
        Ok(())
    }
}

struct MockTranscriptProvider {
    transcript: String,
}

#[async_trait::async_trait]
impl TranscriptProvider for MockTranscriptProvider {
    async fn fetch_transcript(&self, _video_id: &str) -> Result<String, TranscriptError> {
        Ok(self.transcript.clone())
    }
}

struct MockSummarizerAI {
    summary: String,
}

#[async_trait::async_trait]
impl SummarizerAI for MockSummarizerAI {
    async fn summarize_transcript(
        &self,
        _transcript: &str,
        _video_title: &str,
    ) -> Result<String, LLMError> {
        Ok(self.summary.clone())
    }
}

struct MockExaminerAI {
    questions: Vec<MCQuestion>,
}

#[async_trait::async_trait]
impl ExaminerAI for MockExaminerAI {
    async fn generate_mcq(
        &self,
        _video_title: &str,
        _video_description: Option<&str>,
        _video_summary: Option<&str>,
        _num_questions: u8,
        _difficulty: ExamDifficulty,
    ) -> Result<Vec<MCQuestion>, LLMError> {
        Ok(self.questions.clone())
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────

#[test]
fn ingest_local_with_folder_grouping() {
    let scanner = Arc::new(MockScanner::new(vec![
        RawLocalMediaMetadata {
            path: "/videos/module1/vid1.mp4".to_string(),
            title: "Lesson 1 - Intro".to_string(),
            duration_secs: 600,
            subtitles: vec![],
        },
        RawLocalMediaMetadata {
            path: "/videos/module1/vid2.mp4".to_string(),
            title: "Lesson 2 - Basics".to_string(),
            duration_secs: 900,
            subtitles: vec![],
        },
        RawLocalMediaMetadata {
            path: "/videos/module2/vid3.mp4".to_string(),
            title: "Lesson 3 - Advanced".to_string(),
            duration_secs: 1200,
            subtitles: vec![],
        },
    ]));

    let course_repo = Arc::new(InMemoryCourseRepo::new());
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));
    let search_repo = Arc::new(InMemorySearchRepo);

    let use_case = course_pilot::application::use_cases::IngestLocalUseCase::new(
        scanner,
        course_repo.clone(),
        module_repo.clone(),
        video_repo.clone(),
        search_repo,
        None,
        5,
    );

    let input = course_pilot::application::use_cases::IngestLocalInput {
        root_path: "/videos".to_string(),
        course_name: Some("Test Course".to_string()),
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(use_case.execute(input));
    assert!(result.is_ok(), "Ingest should succeed, got: {:?}", result.err());

    let output = result.unwrap();
    assert_eq!(output.videos_count, 3, "Should have 3 total videos");
    assert_eq!(output.modules_count, 2, "Should have 2 modules (one per folder)");

    let courses = course_repo.find_all().unwrap();
    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0].name(), "Test Course");

    let modules = module_repo.find_by_course(courses[0].id()).unwrap();
    assert_eq!(modules.len(), 2);

    let m1_videos = video_repo.find_by_module(modules[0].id()).unwrap();
    assert_eq!(m1_videos.len(), 2, "Module 1 should have 2 videos");

    let m2_videos = video_repo.find_by_module(modules[1].id()).unwrap();
    assert_eq!(m2_videos.len(), 1, "Module 2 should have 1 video");
}

#[test]
fn ingest_playlist_with_mock_fetcher() {
    let fetcher = Arc::new(MockFetcher::new(vec![
        course_pilot::domain::ports::RawVideoMetadata {
            youtube_id: "dQw4w9WgXcQ".to_string(),
            title: "Module 1 - Intro".to_string(),
            description: Some("First video".to_string()),
            duration_secs: 600,
            position: 0,
        },
        course_pilot::domain::ports::RawVideoMetadata {
            youtube_id: "9bZkp7q19f0".to_string(),
            title: "Module 1 - Setup".to_string(),
            description: Some("Second video".to_string()),
            duration_secs: 900,
            position: 1,
        },
    ]));

    let course_repo = Arc::new(InMemoryCourseRepo::new());
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));
    let search_repo = Arc::new(InMemorySearchRepo);

    let use_case = course_pilot::application::use_cases::IngestPlaylistUseCase::new(
        fetcher,
        course_repo.clone(),
        module_repo.clone(),
        video_repo.clone(),
        search_repo,
        None,
        5,
    );

    let input = course_pilot::application::use_cases::IngestPlaylistInput {
        playlist_url: "https://www.youtube.com/playlist?list=PLtest123".to_string(),
        course_name: Some("Test Course".to_string()),
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(use_case.execute(input));
    assert!(result.is_ok(), "Ingest should succeed, got: {:?}", result.err());

    let output = result.unwrap();
    assert_eq!(output.videos_count, 2, "Should have 2 videos");
    assert_eq!(output.modules_count, 1, "Should have 1 module (same major number)");

    let courses = course_repo.find_all().unwrap();
    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0].name(), "Test Course");
}

#[test]
fn ingest_playlist_failure_returns_error() {
    let fetcher = Arc::new(MockFetcher::with_failure());
    let course_repo = Arc::new(InMemoryCourseRepo::new());
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));
    let search_repo = Arc::new(InMemorySearchRepo);

    let use_case = course_pilot::application::use_cases::IngestPlaylistUseCase::new(
        fetcher,
        course_repo.clone(),
        module_repo.clone(),
        video_repo.clone(),
        search_repo,
        None,
        5,
    );

    let input = course_pilot::application::use_cases::IngestPlaylistInput {
        playlist_url: "https://www.youtube.com/playlist?list=PLnonexistent".to_string(),
        course_name: None,
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(use_case.execute(input));
    assert!(result.is_err(), "Should fail for nonexistent playlist");
    match result {
        Err(course_pilot::application::use_cases::IngestError::FetchFailed(msg)) => {
            assert!(msg.to_string().contains("test_not_found"), "Error should contain playlist ID");
        },
        _ => panic!("Expected FetchFailed error, got: {:?}", result),
    }
}

#[test]
fn boundary_detector_integration() {
    let detector = BoundaryDetector::new();
    let titles = vec![
        "1.1 Introduction",
        "1.2 Setup",
        "1.3 Configuration",
        "2.1 Advanced Topics",
        "2.2 Deep Dive",
        "3.1 Conclusion",
    ];

    let groups = detector.group_by_titles(&titles);

    assert_eq!(groups.len(), 3, "Should detect 3 modules from major number changes");
    assert_eq!(groups[0], vec![0, 1, 2], "Module 1: Intro, Setup, Config");
    assert_eq!(groups[1], vec![3, 4], "Module 2: Advanced, Deep Dive");
    assert_eq!(groups[2], vec![5], "Module 3: Conclusion");
}

#[test]
fn ingest_playlist_preserves_labeled_module_boundaries() {
    let fetcher = Arc::new(MockFetcher::new(vec![
        course_pilot::domain::ports::RawVideoMetadata {
            youtube_id: "aaa111aaa11".to_string(),
            title: "Module 1 - Intro".to_string(),
            description: None,
            duration_secs: 100,
            position: 0,
        },
        course_pilot::domain::ports::RawVideoMetadata {
            youtube_id: "bbb222bbb22".to_string(),
            title: "Module 1 - Setup".to_string(),
            description: None,
            duration_secs: 200,
            position: 1,
        },
        course_pilot::domain::ports::RawVideoMetadata {
            youtube_id: "ccc333ccc33".to_string(),
            title: "Module 2 - Deploy".to_string(),
            description: None,
            duration_secs: 300,
            position: 2,
        },
        course_pilot::domain::ports::RawVideoMetadata {
            youtube_id: "ddd444ddd44".to_string(),
            title: "Module 2 - Testing".to_string(),
            description: None,
            duration_secs: 400,
            position: 3,
        },
    ]));

    let course_repo = Arc::new(InMemoryCourseRepo::new());
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));
    let search_repo = Arc::new(InMemorySearchRepo);

    let use_case = course_pilot::application::use_cases::IngestPlaylistUseCase::new(
        fetcher,
        course_repo.clone(),
        module_repo.clone(),
        video_repo.clone(),
        search_repo,
        None,
        5,
    );

    let input = course_pilot::application::use_cases::IngestPlaylistInput {
        playlist_url: "https://www.youtube.com/playlist?list=PLlabeledtest".to_string(),
        course_name: Some("Labeled Test".to_string()),
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(use_case.execute(input));
    assert!(result.is_ok(), "Ingest should succeed: {:?}", result.err());

    let output = result.unwrap();
    assert_eq!(output.videos_count, 4, "Should have 4 videos");
    assert_eq!(output.modules_count, 2, "Should detect 2 modules from labeled patterns");

    let modules = module_repo.find_by_course(&output.course_id).unwrap();
    assert_eq!(modules.len(), 2, "Should persist 2 modules");

    let m1_videos = video_repo.find_by_module(modules[0].id()).unwrap();
    assert_eq!(m1_videos.len(), 2, "Module 1 should have 2 videos");

    let m2_videos = video_repo.find_by_module(modules[1].id()).unwrap();
    assert_eq!(m2_videos.len(), 2, "Module 2 should have 2 videos");
}

#[test]
fn boundary_detector_fallback_batch() {
    let detector = BoundaryDetector::with_batch_size(3);
    let titles =
        vec!["Random topic", "Another topic", "Third topic", "Fourth topic", "Fifth topic"];

    let groups = detector.group_by_titles(&titles);
    assert_eq!(groups.len(), 2, "Should fallback to batch=3");
    assert_eq!(groups[0].len(), 3);
    assert_eq!(groups[1].len(), 2);
}

#[test]
fn transcript_chunker_integration() {
    let chunker = TranscriptChunker::new();

    let short = "This is a short transcript.";
    assert_eq!(chunker.chunk(short).len(), 1);

    let long = "sentence. ".repeat(2000);
    let chunks = chunker.chunk(&long);
    assert!(chunks.len() > 1, "Long text should be chunked into {} parts", chunks.len());

    assert_eq!(chunker.chunk(&long), chunker.chunk(&long));
}

#[test]
fn test_in_memory_video_repo_find_by_course() {
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));

    let course_id1 = CourseId::new();
    let course_id2 = CourseId::new();

    let m1 = Module::new(ModuleId::new(), course_id1, "Module 1".to_string(), 2);
    let m2 = Module::new(ModuleId::new(), course_id1, "Module 2".to_string(), 1);
    let m3 = Module::new(ModuleId::new(), course_id2, "Module 3".to_string(), 1);

    module_repo.save(&m1).unwrap();
    module_repo.save(&m2).unwrap();
    module_repo.save(&m3).unwrap();

    let v1 = Video::new(
        VideoId::new(),
        *m1.id(),
        VideoSource::local_path("/videos/vid1.mp4").unwrap(),
        "Video 1".to_string(),
        100,
        2,
    );
    let v2 = Video::new(
        VideoId::new(),
        *m1.id(),
        VideoSource::local_path("/videos/vid2.mp4").unwrap(),
        "Video 2".to_string(),
        100,
        1,
    );
    let v3 = Video::new(
        VideoId::new(),
        *m2.id(),
        VideoSource::local_path("/videos/vid3.mp4").unwrap(),
        "Video 3".to_string(),
        100,
        1,
    );
    let v4 = Video::new(
        VideoId::new(),
        *m3.id(),
        VideoSource::local_path("/videos/vid4.mp4").unwrap(),
        "Video 4".to_string(),
        100,
        1,
    );

    video_repo.save(&v1).unwrap();
    video_repo.save(&v2).unwrap();
    video_repo.save(&v3).unwrap();
    video_repo.save(&v4).unwrap();

    // Query for course_id1.
    // Course 1 has m1 (sort_order 2) and m2 (sort_order 1).
    // So the videos from m2 should come first, then videos from m1.
    // Within m1, v2 (sort_order 1) should come before v1 (sort_order 2).
    // So the expected order is: v3, v2, v1. v4 should be filtered out.
    let videos = video_repo.find_by_course(&course_id1).unwrap();
    assert_eq!(videos.len(), 3);
    assert_eq!(videos[0].id(), v3.id());
    assert_eq!(videos[1].id(), v2.id());
    assert_eq!(videos[2].id(), v1.id());
}

#[test]
fn test_summarize_video_use_case() {
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));

    let module_id = ModuleId::new();
    let course_id = CourseId::new();
    let module = Module::new(module_id, course_id, "Test Module".to_string(), 1);
    module_repo.save(&module).unwrap();

    let video_id = VideoId::new();
    let yt_id = YouTubeVideoId::new("dQw4w9WgXcQ").unwrap();
    let video = Video::new(
        video_id,
        module_id,
        VideoSource::youtube(yt_id),
        "Test Video Title".to_string(),
        600,
        1,
    );
    video_repo.save(&video).unwrap();

    let mock_transcript = "This is a mock transcript of the video.".to_string();
    let mock_summary = "This is a mock summary.".to_string();

    let transcript_provider =
        Arc::new(MockTranscriptProvider { transcript: mock_transcript.clone() });
    let summarizer_ai = Arc::new(MockSummarizerAI { summary: mock_summary.clone() });

    let use_case = SummarizeVideoUseCase::new(
        summarizer_ai.clone(),
        transcript_provider.clone(),
        video_repo.clone(),
    );

    let input = SummarizeVideoInput { video_id, force_refresh: false };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(use_case.execute(input));
    assert!(result.is_ok(), "Summarize should succeed, got: {:?}", result.err());

    let output: SummarizeVideoOutput = result.unwrap();
    assert_eq!(output.summary, mock_summary);
    assert_eq!(output.transcript_used, mock_transcript);
    assert!(!output.cached);

    // Verify video in video_repo is updated to include transcript and summary
    let updated_video = video_repo.find_by_id(&video_id).unwrap().unwrap();
    assert_eq!(updated_video.transcript(), Some(mock_transcript.as_str()));
    assert_eq!(updated_video.summary(), Some(mock_summary.as_str()));

    // Running again with force_refresh = false returns cached: true
    let input_second = SummarizeVideoInput { video_id, force_refresh: false };
    let result_second =
        tokio::runtime::Runtime::new().unwrap().block_on(use_case.execute(input_second));
    assert!(result_second.is_ok());
    let output_second: SummarizeVideoOutput = result_second.unwrap();
    assert_eq!(output_second.summary, mock_summary);
    assert_eq!(output_second.transcript_used, mock_transcript);
    assert!(output_second.cached);
}

#[test]
fn test_take_exam_use_case_flow() {
    let module_repo = Arc::new(InMemoryModuleRepo::new());
    let video_repo = Arc::new(InMemoryVideoRepo::new(module_repo.clone()));
    let exam_repo = Arc::new(InMemoryExamRepo::new());

    let module_id = ModuleId::new();
    let course_id = CourseId::new();
    let module = Module::new(module_id, course_id, "Test Module".to_string(), 1);
    module_repo.save(&module).unwrap();

    let video_id = VideoId::new();
    let video = Video::new(
        video_id,
        module_id,
        VideoSource::local_path("/videos/test.mp4").unwrap(),
        "Test Video Title".to_string(),
        600,
        1,
    );
    video_repo.save(&video).unwrap();
    assert!(!video.is_completed());

    let q1 = MCQuestion {
        question: "Question 1".to_string(),
        options: vec!["Option A".to_string(), "Option B".to_string()],
        correct_index: 1,
        explanation: "Explanation 1".to_string(),
    };
    let q2 = MCQuestion {
        question: "Question 2".to_string(),
        options: vec!["Option A".to_string(), "Option B".to_string()],
        correct_index: 0,
        explanation: "Explanation 2".to_string(),
    };

    let examiner_ai = Arc::new(MockExaminerAI { questions: vec![q1, q2] });

    let use_case = TakeExamUseCase::new(examiner_ai.clone(), video_repo.clone(), exam_repo.clone());

    let generate_input =
        GenerateExamInput { video_id, num_questions: 2, difficulty: ExamDifficulty::Medium };

    let gen_result =
        tokio::runtime::Runtime::new().unwrap().block_on(use_case.generate(generate_input));
    assert!(gen_result.is_ok(), "Generate should succeed, got: {:?}", gen_result.err());

    let gen_output = gen_result.unwrap();
    let exam_id = gen_output.exam_id;
    assert_eq!(gen_output.questions.len(), 2);

    // Assert that the exam is saved under the correct video ID
    let exams_by_video = exam_repo.find_by_video(&video_id).unwrap();
    assert_eq!(exams_by_video.len(), 1);
    assert_eq!(exams_by_video[0].id(), &exam_id);

    // Call submit with correct answers vec![1, 0]
    let submit_input = SubmitExamInput { exam_id, answers: vec![1, 0] };

    let submit_result = use_case.submit(submit_input);
    assert!(submit_result.is_ok(), "Submit should succeed, got: {:?}", submit_result.err());

    let submit_output = submit_result.unwrap();
    assert_eq!(submit_output.score, 1.0);
    assert!(submit_output.passed);
    assert!(submit_output.video_marked_complete);

    // Assert: The video in video_repo is now marked as completed!
    let updated_video = video_repo.find_by_id(&video_id).unwrap().unwrap();
    assert!(updated_video.is_completed());
}
