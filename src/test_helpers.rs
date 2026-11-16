//! Test Helpers
//!
//! The responsibility of the `test_helpers` crate  is to expose test helper
//! functions.
//! They allow tests to focus on function behaviour instead of heavy setup.

use crate::models;

/// Models the scalability_001 question from the question bank.
///
/// This is an exact 1 for 1 of the scalability_001 in the JSON question bank.
pub fn scalability_question() -> models::Question {
    models::Question {
        id: "scalability_001".to_string(),
        prompt: "A service is scalable if...".to_string(),
        options: [
            "It can handle increased workload by adding resources while maintaining acceptable performance".to_string(),
            "It responds quickly to requests when the workload is low".to_string(),
            "It continues operating when an individual server fails".to_string(),
            "It automatically restarts after a software crash".to_string(),
        ],
        correct_answer: models::Choice::A,
        explanation: "Scalability is a service's ability to handle growing workload by adding resources, such as CPU, memory, or additional servers, while maintaining acceptable performance. For example, adding servers might allow it to support more concurrent users without unacceptable increases in response time. The improvement does not have to be exactly proportional to the resources added; that stronger property is called linear scalability.".to_string(),
    }
}

/// Models the cap_theorem_001 question from the question bank.
///
/// This is an exact 1 for 1 of the cap_theorem_001 in the JSON question bank.
pub fn cap_theorem_question() -> models::Question {
    models::Question {
        id: "cap_theorem_001".to_string(),
        prompt: "According to the CAP theorem, what does consistency mean?".to_string(),
        options: [
            "Every request receives a response, even if it is stale".to_string(),
            "The system continues operating across a network partition".to_string(),
            "Every read receives the most recent write or an error".to_string(),
            "All replicas eventually converge if writes stop".to_string(),
        ],
        correct_answer: models::Choice::C,
        explanation: "In CAP, consistency means every read observes the most recent write or receives an error. Availability and partition tolerance describe different guarantees.".to_string(),
    }
}
