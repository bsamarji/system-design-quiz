//! Terminal
//!
//! The responsibility of the `terminal` crate is to display questions/results
//! and to capture user input.

use std::io::{self, Write};

use crate::{
    app_state::{self, Screen},
    models::Choice,
    quiz::{QuizScore, calculate_score, evaluate_answer, topics_to_review},
};
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use textwrap::wrap;

/// This struct can be used to enter and leave the alternate screen.
///
/// The struct only has a constructor and destructor. The destructor
/// implements the `Drop` trait and is automatically called when the
/// struct goes out of scope.
pub struct TerminalGuard;

impl TerminalGuard {
    /// Constructor for the `TerminalGuard`.
    ///
    /// Enables raw mode and switches to the alternate screen.
    pub fn enter<W: Write>(mut out: W) -> io::Result<Self> {
        enable_raw_mode()?;

        if let Err(err) = execute!(out, EnterAlternateScreen) {
            disable_raw_mode()?;
            return Err(err);
        }

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    /// Destructor for the `TerminalGuard`.
    ///
    /// Implements the drop trait for `TerminalGuard`.
    ///
    /// This destructor disables raw mode and leaves the alternate screen.
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// Retrieve the current terminal width.
///
/// This function places an upper limit of 120 columns and
/// a lower limit of 20 columns.
///
/// This function also defaults to 80 columns if the terminal size cannot be
/// found.
fn terminal_content_width() -> usize {
    crossterm::terminal::size()
        .map(|(width, _)| usize::from(width))
        .unwrap_or(80)
        .saturating_sub(4)
        .clamp(20, 120)
}

/// Helper function to wrap text when printing to stdout
fn print_wrapped<W: Write>(stdout: &mut W, text: &str, width: usize) -> io::Result<()> {
    for line in wrap(text, width) {
        queue!(stdout, Print(line), Print("\r\n"))?;
    }

    Ok(())
}

/// Helper function to wrap text when printing answer options to stdout
///
/// This function handles indentation as well as text wrapping.
fn print_wrapped_option<W: Write>(
    stdout: &mut W,
    label: char,
    option: &str,
    width: usize,
) -> io::Result<()> {
    let prefix = format!("  {label}. ");
    let indentation = " ".repeat(prefix.len());

    let option_width = width.saturating_sub(prefix.len()).max(1);

    for (index, line) in wrap(option, option_width).iter().enumerate() {
        if index == 0 {
            queue!(
                stdout,
                SetForegroundColor(Color::Cyan),
                Print(&prefix),
                SetAttribute(Attribute::Reset),
            )?;
        } else {
            queue!(stdout, Print(&indentation))?;
        }

        queue!(stdout, Print(line), Print("\r\n"),)?;
    }

    Ok(())
}

/// Renders a `Screen` in the terminal's alternate screen.
///
/// The `Screen` is auto-formatted using the display functions.
///
/// The `Screen` can be of Question, Feedback or Summary, depending on the app
/// state.
pub fn render<W: Write>(app: &mut app_state::App, mut stdout: W) -> io::Result<()> {
    let width = terminal_content_width();

    match app.screen {
        Screen::Question => {
            display_question(app, &mut stdout, width)?;
            display_question_navigation(&mut stdout, width)?;
        }
        Screen::Feedback => {
            display_question(app, &mut stdout, width)?;
            let is_correct = evaluate_answer(app);
            display_feedback(is_correct, app, &mut stdout, width)?;
            display_feedback_navigation(&mut stdout, width)?;
        }
        Screen::Summary => {
            let quiz_score = calculate_score(&app.questions, &app.answers);
            let topics = topics_to_review(&app.questions, &app.answers);
            display_summary(quiz_score, &mut stdout, topics)?;
            display_summary_navigation(&mut stdout, width)?;
        }
        Screen::Unanswered => {
            display_unanswered(app, &mut stdout, width)?;
            display_unanswered_navigation(&mut stdout, width)?;
        }
    }

    Ok(())
}

/// Prints a question to stdout.
///
/// The question is formatted into a user-friendly quiz style.
fn display_question<W: Write>(app: &app_state::App, mut stdout: W, width: usize) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0),)?;

    let current_question_number = app.current_question_index + 1;
    let total_question_count = app.questions.len();

    queue!(stdout, SetAttribute(Attribute::Bold))?;
    print_wrapped(
        &mut stdout,
        &format!(
            "Question {} of {}",
            current_question_number, total_question_count
        ),
        width,
    )?;
    queue!(stdout, SetAttribute(Attribute::Reset), Print("\r\n"),)?;

    let question = &app.questions[app.current_question_index];

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetAttribute(Attribute::Underlined)
    )?;
    print_wrapped(&mut stdout, &format!("Question ID: {}", question.id), width)?;
    queue!(stdout, SetAttribute(Attribute::Reset), Print("\r\n"))?;

    queue!(stdout, SetAttribute(Attribute::Bold))?;

    print_wrapped(&mut stdout, &question.prompt, width)?;
    queue!(stdout, SetAttribute(Attribute::Reset), Print("\r\n"),)?;

    for (label, option) in ['A', 'B', 'C', 'D'].iter().zip(question.options.iter()) {
        print_wrapped_option(&mut stdout, *label, option, width)?;
        queue!(stdout, Print("\r\n"))?;
    }

    stdout.flush()
}

/// Prints feedback to stdout
///
/// A little message is displayed along with the correct answer and explanation
fn display_feedback<W: Write>(
    is_correct: bool,
    app: &mut app_state::App,
    mut stdout: W,
    width: usize,
) -> io::Result<()> {
    let current_question = &app.questions[app.current_question_index];
    let correct_choice = match current_question.correct_answer {
        Choice::A => 'A',
        Choice::B => 'B',
        Choice::C => 'C',
        Choice::D => 'D',
    };
    let user_choice = match app.answers[&current_question.id] {
        Choice::A => 'A',
        Choice::B => 'B',
        Choice::C => 'C',
        Choice::D => 'D',
    };

    queue!(stdout, Print("-".repeat(30)), Print("\r\n\r\n"),)?;

    if is_correct {
        queue!(
            stdout,
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Green),
            Print("CORRECT"),
            SetAttribute(Attribute::Reset),
            Print("\r\n\r\n"),
        )?;
    } else {
        queue!(
            stdout,
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::Red),
            Print("INCORRECT"),
            SetAttribute(Attribute::Reset),
            Print("\r\n\r\n"),
        )?;
    }

    queue!(
        stdout,
        Print(format!("• Your answer: {}", user_choice)),
        Print("\r\n\r\n"),
        Print(format!("• Correct answer: {}", correct_choice)),
        Print("\r\n\r\n"),
    )?;

    queue!(stdout, Print("-".repeat(30)), Print("\r\n\r\n"),)?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        Print("Explanation:"),
        Print("\r\n\r\n"),
        SetAttribute(Attribute::Reset)
    )?;
    print_wrapped(&mut stdout, &current_question.explanation, width)?;
    queue!(stdout, Print("\r\n"),)?;

    stdout.flush()
}

/// Prints the quiz summary to stdout
fn display_summary<W: Write>(
    score: QuizScore,
    mut stdout: W,
    topics: Vec<String>,
) -> io::Result<()> {
    let grade_color = if score.grade == 'A' {
        Color::DarkGreen
    } else if score.grade == 'B' {
        Color::Green
    } else if score.grade == 'C' {
        Color::Yellow
    } else if score.grade == 'D' {
        Color::DarkYellow
    } else if score.grade == 'E' {
        Color::Red
    } else {
        Color::DarkRed
    };

    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0),)?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetAttribute(Attribute::Underlined),
        Print("Summary"),
        SetAttribute(Attribute::Reset),
        Print("\r\n\r\n"),
    )?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        Print(format!("• Total score: {}/{}", score.correct, score.total)),
        SetAttribute(Attribute::Reset),
        Print("\r\n"),
    )?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        Print(format!("• Percentage: {:.1}%", score.percentage)),
        SetAttribute(Attribute::Reset),
        Print("\r\n"),
    )?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        Print("• Grade: "),
        SetForegroundColor(grade_color),
        Print(score.grade),
        SetAttribute(Attribute::Reset),
        Print("\r\n\r\n"),
    )?;

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        Print("Topics to review:"),
        SetAttribute(Attribute::Reset),
        Print("\r\n"),
    )?;

    if !topics.is_empty() {
        for topic in topics {
            queue!(stdout, Print(format!("• {}", topic)), Print("\r\n"))?;
        }
    }

    queue!(stdout, Print("\r\n"))?;

    stdout.flush()
}

/// Prints the number of unanswered questions to the screen and the id of the
/// first question that remains unanswered
fn display_unanswered<W: Write>(
    app: &app_state::App,
    mut stdout: W,
    width: usize,
) -> io::Result<()> {
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0),)?;

    let missing_answer_count = app.questions.len() - app.answers.len();

    queue!(stdout, SetAttribute(Attribute::Bold))?;
    print_wrapped(
        &mut stdout,
        &format!("{} questions remain unanswered.", missing_answer_count),
        width,
    )?;
    queue!(stdout, SetAttribute(Attribute::Reset))?;

    for question in &app.questions {
        if !app.answers.contains_key(&question.id) {
            print_wrapped(
                &mut stdout,
                &format!("Next unanswered question: {}", question.id),
                width,
            )?;
            queue!(stdout, Print("\r\n"),)?;
            break;
        }
    }

    stdout.flush()
}

/// Prints the actionable keys available to the user on the `Question` screen
fn display_question_navigation<W: Write>(mut stdout: W, width: usize) -> io::Result<()> {
    queue!(stdout, SetAttribute(Attribute::Bold))?;
    print_wrapped(
        &mut stdout,
        "Press [A] - [D] Answer  [N] Next  [P] Previous  [Q] Quit",
        width,
    )?;

    stdout.flush()
}

/// Prints the actionable keys available to the user on the `Feedback` screen
/// or the `Summary` screen
fn display_feedback_navigation<W: Write>(mut stdout: W, width: usize) -> io::Result<()> {
    queue!(stdout, SetAttribute(Attribute::Bold))?;
    print_wrapped(&mut stdout, "Press [N] Next  [P] Previous  [Q] Quit", width)?;

    stdout.flush()
}

/// Prints the actionable keys available to the user on the `Summary` screen
fn display_summary_navigation<W: Write>(mut stdout: W, width: usize) -> io::Result<()> {
    queue!(stdout, SetAttribute(Attribute::Bold))?;
    print_wrapped(
        &mut stdout,
        "Press [P] Previous  [R] Reset  [Q] Quit",
        width,
    )?;

    stdout.flush()
}

/// Prints the actionable keys available to the user on the `Unanswered` screen
fn display_unanswered_navigation<W: Write>(mut stdout: W, width: usize) -> io::Result<()> {
    queue!(stdout, SetAttribute(Attribute::Bold))?;
    print_wrapped(
        &mut stdout,
        "Press [U] to go the next unanswered question  [P] Previous  [Q] Quit",
        width,
    )?;

    stdout.flush()
}

#[cfg(test)]
mod tests {
    use crossterm::terminal::is_raw_mode_enabled;
    use std::io::IsTerminal;

    use super::*;

    #[test]
    fn terminal_guard_manages_raw_mode_when_a_tty_is_available() {
        if !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
            eprintln!("Skipping terminal guard test: no interactive terminal available");
            return;
        }

        {
            let mut buffer = Vec::new();
            let terminal = TerminalGuard::enter(&mut buffer);
            assert!(terminal.is_ok());

            let is_enabled = is_raw_mode_enabled().expect("Error retrieving raw mode status");
            assert!(is_enabled, "Terminal guard entry should enable raw mode");

            let mut expected = Vec::new();
            execute!(expected, EnterAlternateScreen).unwrap();

            assert_eq!(
                buffer, expected,
                "Terminal guard entry should enter alternate screen"
            );
        }
        // Drop currently writes to real stdout, so testing for LeaveAlternateScreen
        // is not viable until TerminalGuard is refactored
        let is_enabled = is_raw_mode_enabled().expect("Error retrieving raw mode status");
        assert!(!is_enabled, "Terminal guard drop should disable raw mode");
    }
}
