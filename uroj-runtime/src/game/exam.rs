use std::collections::HashMap;
use chrono::Duration;

use super::instance::InstanceBase;

struct ExamInstance<'a> {
    instance: InstanceBase<'a>,
    exam_id: String,
    duration: Duration,
}

struct ExamManager<'a> {
    exams: HashMap<String, ExamInstance<'a>>,
    
}