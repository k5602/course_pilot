use glib::subclass::prelude::*;

mod course_imp {
    use std::cell::RefCell;

    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct CourseObject {
        pub id: RefCell<String>,
        pub title: RefCell<String>,
        pub description: RefCell<Option<String>>,
        pub module_count: RefCell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CourseObject {
        const NAME: &'static str = "CourseObject";
        type Type = super::CourseObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for CourseObject {}
}

glib::wrapper! {
    pub struct CourseObject(ObjectSubclass<course_imp::CourseObject>);
}

impl CourseObject {
    pub fn new(id: String, title: String, description: Option<String>, module_count: i32) -> Self {
        let obj: CourseObject = glib::Object::builder().build();
        let imp = obj.imp();
        *imp.id.borrow_mut() = id;
        *imp.title.borrow_mut() = title;
        *imp.description.borrow_mut() = description;
        *imp.module_count.borrow_mut() = module_count;
        obj
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn description(&self) -> Option<String> {
        self.imp().description.borrow().clone()
    }

    pub fn module_count(&self) -> i32 {
        *self.imp().module_count.borrow()
    }
}

mod quiz_imp {
    use std::cell::RefCell;

    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct QuizObject {
        pub id: RefCell<String>,
        pub title: RefCell<String>,
        pub video_id: RefCell<String>,
        pub is_taken: RefCell<bool>,
        pub score: RefCell<Option<f32>>,
        pub passed: RefCell<Option<bool>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for QuizObject {
        const NAME: &'static str = "QuizObject";
        type Type = super::QuizObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for QuizObject {}
}

glib::wrapper! {
    pub struct QuizObject(ObjectSubclass<quiz_imp::QuizObject>);
}

impl QuizObject {
    pub fn new(
        id: String,
        title: String,
        video_id: String,
        is_taken: bool,
        score: Option<f32>,
        passed: Option<bool>,
    ) -> Self {
        let obj: QuizObject = glib::Object::builder().build();
        let imp = obj.imp();
        *imp.id.borrow_mut() = id;
        *imp.title.borrow_mut() = title;
        *imp.video_id.borrow_mut() = video_id;
        *imp.is_taken.borrow_mut() = is_taken;
        *imp.score.borrow_mut() = score;
        *imp.passed.borrow_mut() = passed;
        obj
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn is_taken(&self) -> bool {
        *self.imp().is_taken.borrow()
    }

    pub fn score(&self) -> Option<f32> {
        *self.imp().score.borrow()
    }

    pub fn passed(&self) -> Option<bool> {
        *self.imp().passed.borrow()
    }
}

mod video_row_imp {
    use std::cell::RefCell;

    use glib::subclass::prelude::*;

    #[derive(Default)]
    pub struct VideoRowObject {
        pub id: RefCell<String>,
        pub module_id: RefCell<String>,
        pub title: RefCell<String>,
        pub is_completed: RefCell<bool>,
        pub duration_secs: RefCell<u32>,
        pub source_type: RefCell<String>,
        pub sort_order: RefCell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoRowObject {
        const NAME: &'static str = "VideoRowObject";
        type Type = super::VideoRowObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for VideoRowObject {}
}

glib::wrapper! {
    pub struct VideoRowObject(ObjectSubclass<video_row_imp::VideoRowObject>);
}

impl VideoRowObject {
    pub fn new(
        id: String,
        module_id: String,
        title: String,
        is_completed: bool,
        duration_secs: u32,
        source_type: String,
        sort_order: u32,
    ) -> Self {
        let obj: VideoRowObject = glib::Object::builder().build();
        let imp = obj.imp();
        *imp.id.borrow_mut() = id;
        *imp.module_id.borrow_mut() = module_id;
        *imp.title.borrow_mut() = title;
        *imp.is_completed.borrow_mut() = is_completed;
        *imp.duration_secs.borrow_mut() = duration_secs;
        *imp.source_type.borrow_mut() = source_type;
        *imp.sort_order.borrow_mut() = sort_order;
        obj
    }

    pub fn id(&self) -> String {
        self.imp().id.borrow().clone()
    }

    pub fn module_id(&self) -> String {
        self.imp().module_id.borrow().clone()
    }

    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
    }

    pub fn is_completed(&self) -> bool {
        *self.imp().is_completed.borrow()
    }

    pub fn duration_secs(&self) -> u32 {
        *self.imp().duration_secs.borrow()
    }

    pub fn source_type(&self) -> String {
        self.imp().source_type.borrow().clone()
    }

    pub fn sort_order(&self) -> u32 {
        *self.imp().sort_order.borrow()
    }
}
