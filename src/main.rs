use std::{
    io::{self, stdout, Write},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, size},
};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Open the customization menu
    #[arg(short, long)]
    menu: bool,
}


#[derive(Debug, Clone, Copy, PartialEq)]
enum TimerStyle {
    Text,
    Boxes,
    BoxesRounded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RunOption {
    Default,
    Hold,
}

#[derive(Debug, Clone, Copy)]
struct AppConfig {
    style: TimerStyle,
    run_option: RunOption,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            style: TimerStyle::Boxes,
            run_option: RunOption::Default,
        }
    }
}

fn draw_box(stdout: &mut io::Stdout, x: u16, y: u16, width: u16, height: u16, style: TimerStyle) -> io::Result<()> {
    let (top_left, top_right, bottom_left, bottom_right, horizontal, vertical) = match style {
        TimerStyle::Text => return Ok(()), // No box for text style
        TimerStyle::Boxes => ('┌', '┐', '└', '┘', '─', '│'),
        TimerStyle::BoxesRounded => ('╭', '╮', '╰', '╯', '─', '│'),
    };

    // Top border
    execute!(stdout, crossterm::cursor::MoveTo(x, y))?;
    print!("{}", top_left);
    for _ in 0..width - 2 {
        print!("{}", horizontal);
    }
    print!("{}", top_right);

    // Sides
    for i in 1..height - 1 {
        execute!(stdout, crossterm::cursor::MoveTo(x, y + i))?;
        print!("{}", vertical);
        execute!(stdout, crossterm::cursor::MoveTo(x + width - 1, y + i))?;
        print!("{}", vertical);
    }

    // Bottom border
    execute!(stdout, crossterm::cursor::MoveTo(x, y + height - 1))?;
    print!("{}", bottom_left);
    for _ in 0..width - 2 {
        print!("{}", horizontal);
    }
    print!("{}", bottom_right);
    Ok(())
}

fn run_menu(config: &mut AppConfig) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide, Clear(ClearType::All))?;

    let mut selected_option = 0;
    let options = [
        "Style",
        "Run Option",
        "Start Timer",
        "Exit",
    ];

    loop {
        let (cols, rows) = terminal::size()?;
        execute!(stdout, Clear(ClearType::All))?;

        let title = "Customization Menu";
        let title_x = (cols.saturating_sub(title.len() as u16)) / 2;
        execute!(stdout, crossterm::cursor::MoveTo(title_x, 1))?;
        print!("{}", title);

        for (i, option) in options.iter().enumerate() {
            let y = 3 + i as u16;
            execute!(stdout, crossterm::cursor::MoveTo(cols / 2 - 10, y))?;
            if i == selected_option {
                execute!(stdout, SetBackgroundColor(Color::White), SetForegroundColor(Color::Black))?;
            }
            print!("{}", option);
            execute!(stdout, ResetColor)?;

            match i {
                0 => { // Style
                    print!(": {:?}", config.style);
                }
                1 => { // Run Option
                    print!(": {:?}", config.run_option);
                }
                _ => {}
            }
        }
        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Up => {
                    if selected_option > 0 {
                        selected_option -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_option < options.len() - 1 {
                        selected_option += 1;
                    }
                }
                KeyCode::Enter => {
                    match selected_option {
                        0 => { // Style
                            config.style = match config.style {
                                TimerStyle::Text => TimerStyle::Boxes,
                                TimerStyle::Boxes => TimerStyle::BoxesRounded,
                                TimerStyle::BoxesRounded => TimerStyle::Text,
                            };
                        }
                        1 => { // Run Option
                            config.run_option = match config.run_option {
                                RunOption::Default => RunOption::Hold,
                                RunOption::Hold => RunOption::Default,
                            };
                        }
                        2 => { // Start Timer
                            disable_raw_mode()?;
                            execute!(stdout, Show, Clear(ClearType::All))?;
                            return Ok(());
                        }
                        3 => { // Exit
                            disable_raw_mode()?;
                            execute!(stdout, Show, Clear(ClearType::All))?;
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    execute!(stdout, Show, Clear(ClearType::All))?;
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    }
}

fn get_ao(results: &[Duration], count: usize) -> Option<Duration> {
    if results.len() < count {
        return None;
    }
    let relevant_results: Vec<Duration> = results.iter().rev().take(count).cloned().collect();
    if relevant_results.len() < 3 { // Need at least 3 to drop best/worst
        let sum: Duration = relevant_results.iter().sum();
        return Some(sum / relevant_results.len() as u32);
    }

    let mut sorted_results = relevant_results;
    sorted_results.sort();

    // Drop best and worst
    let sum: Duration = sorted_results[1..sorted_results.len() - 1].iter().sum();
    Some(sum / (sorted_results.len() - 2) as u32)
}

fn get_best_of(results: &[Duration], count: usize) -> Option<Duration> {
    if results.len() == 0 {
        return None;
    }
    results.iter().rev().take(count).min().cloned()
}

fn get_best_total(results: &[Duration]) -> Option<Duration> {
    if results.len() == 0 {
        return None;
    }
    results.iter().min().cloned()
}

fn run_timer(config: AppConfig) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide, Clear(ClearType::All))?;

    let mut timer_running = false;
    let mut start_time: Option<Instant> = None;
    let mut last_duration: Option<Duration> = None;
    let mut results: Vec<Duration> = Vec::new();

    // State for Hold run option
    let mut space_is_down = false;
    let mut armed_and_ready = false;
    let mut hold_start_instant: Option<Instant> = None;
    let hold_duration_threshold = Duration::from_millis(500);

    loop {
        let (cols, rows) = terminal::size()?;

        // Clear the screen for each frame
        execute!(stdout, Clear(ClearType::All))?;

        // Draw results table
        let table_start_x = 0;
        let table_start_y = 0;
        let max_table_rows = (rows as usize).saturating_sub(5); // Leave space for instructions and timer

        // Table Header
        execute!(stdout, crossterm::cursor::MoveTo(table_start_x, table_start_y))?;
        print!("ATTEMPT | TIME");

        for (i, duration) in results.iter().rev().take(max_table_rows).enumerate() {
            let display_row = table_start_y + 1 + i as u16; // +1 for header
            execute!(stdout, crossterm::cursor::MoveTo(table_start_x, display_row))?;
            print!("{: <7} | {:.3}s", results.len() - i, duration.as_secs_f32());
        }

        // Calculate and display statistics
        let stats_start_y = table_start_y + 1 + results.len().min(max_table_rows) as u16 + 1; // Below table + 1 line spacing
        let stats_x = 0;

        execute!(stdout, crossterm::cursor::MoveTo(stats_x, stats_start_y))?;
        if let Some(ao5) = get_ao(&results, 5) {
            print!("Ao5: {:.3}s", ao5.as_secs_f32());
        }

        execute!(stdout, crossterm::cursor::MoveTo(stats_x, stats_start_y + 1))?;
        if let Some(ao12) = get_ao(&results, 12) {
            print!("Ao12: {:.3}s", ao12.as_secs_f32());
        }

        execute!(stdout, crossterm::cursor::MoveTo(stats_x, stats_start_y + 2))?;
        if let Some(bo12) = get_best_of(&results, 12) {
            print!("Bo12: {:.3}s", bo12.as_secs_f32());
        }

        execute!(stdout, crossterm::cursor::MoveTo(stats_x, stats_start_y + 3))?;
        if let Some(best_total) = get_best_total(&results) {
            print!("Best: {:.3}s", best_total.as_secs_f32());
        }

        // Display instructions
        let instructions = "Press Space to start/stop. Press Esc to exit.";
        let inst_x = (cols as usize).saturating_sub(instructions.len()) / 2;
        execute!(stdout, crossterm::cursor::MoveTo(inst_x as u16, rows - 3))?;
        print!("{}", instructions);

        // Timer container dimensions
        let timer_box_width = 20;
        let timer_box_height = 3;
        let timer_box_x = (cols.saturating_sub(timer_box_width)) / 2;
        let timer_box_y = (rows.saturating_sub(timer_box_height + 3)) / 2; // +3 for indicator bar and some spacing

        if config.style != TimerStyle::Text {
            draw_box(&mut stdout, timer_box_x, timer_box_y, timer_box_width, timer_box_height, config.style)?;
        }

        // Display timer
        let time_str = if timer_running {
            if let Some(start) = start_time {
                format!("{:.3}s", start.elapsed().as_secs_f32())
            } else {
                "0.000s".to_string()
            }
        } else if let Some(duration) = last_duration {
            format!("{:.3}s", duration.as_secs_f32())
        } else {
            "0.000s".to_string()
        };

        let (timer_text_x, timer_text_y) = if config.style != TimerStyle::Text {
            (timer_box_x + (timer_box_width.saturating_sub(time_str.len() as u16)) / 2, timer_box_y + 1)
        } else {
            ((cols.saturating_sub(time_str.len() as u16)) / 2, rows / 2 - 1)
        };
        execute!(stdout, crossterm::cursor::MoveTo(timer_text_x, timer_text_y))?;
        print!("{}", time_str);

        // Indicator bar container dimensions
        let indicator_box_width = timer_box_width;
        let indicator_box_height = 3;
        let indicator_box_x = timer_box_x;
        let indicator_box_y = timer_box_y + timer_box_height + 1; // 1 line spacing

        if config.style != TimerStyle::Text {
            draw_box(&mut stdout, indicator_box_x, indicator_box_y, indicator_box_width, indicator_box_height, config.style)?;
        }

        // Indicator bar content
        let bar_color = if timer_running {
            Color::Red
        } else if armed_and_ready {
            Color::Yellow
        } else {
            Color::Green
        };
        let status_text = if timer_running { "RUNNING" } else if armed_and_ready { "ARMED" } else { "IDLE" };
        let (status_text_x, status_text_y) = if config.style != TimerStyle::Text {
            (indicator_box_x + (indicator_box_width.saturating_sub(status_text.len() as u16)) / 2, indicator_box_y + 1)
        } else {
            ((cols.saturating_sub(status_text.len() as u16)) / 2, rows / 2 + 1)
        };

        execute!(
            stdout,
            SetBackgroundColor(bar_color),
            SetForegroundColor(Color::Black),
            crossterm::cursor::MoveTo(status_text_x, status_text_y)
        )?;
        print!("{}", status_text);
        execute!(stdout, ResetColor)?;
        stdout.flush()?;

        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(' ') => {
                        if config.run_option == RunOption::Hold {
                            if key_event.kind == event::KeyEventKind::Press {
                                if !timer_running && !space_is_down {
                                    space_is_down = true;
                                    hold_start_instant = Some(Instant::now());
                                    armed_and_ready = false; // Reset arming on new press
                                } else if timer_running {
                                    // Stop timer on press
                                    timer_running = false;
                                    if let Some(start) = start_time {
                                        let duration = start.elapsed();
                                        last_duration = Some(duration);
                                        results.push(duration);
                                    }
                                    space_is_down = false;
                                    hold_start_instant = None;
                                    armed_and_ready = false;
                                }
                            } else if key_event.kind == event::KeyEventKind::Release {
                                space_is_down = false;
                                if armed_and_ready {
                                    // Start timer on release after arming
                                    start_time = Some(Instant::now());
                                    timer_running = true;
                                    armed_and_ready = false;
                                    hold_start_instant = None;
                                } else if !timer_running {
                                    // If not armed and not running, just reset hold state (quick press/release)
                                    hold_start_instant = None;
                                }
                            }
                        } else { // Default run option
                            if key_event.kind == event::KeyEventKind::Press {
                                if !timer_running {
                                    // Start timer
                                    start_time = Some(Instant::now());
                                    timer_running = true;
                                } else {
                                    // Stop timer
                                    timer_running = false;
                                    if let Some(start) = start_time {
                                        let duration = start.elapsed();
                                        last_duration = Some(duration);
                                        results.push(duration);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Check for hold duration if space is pressed and not running (for Hold option)
        if config.run_option == RunOption::Hold && space_is_down && !timer_running && hold_start_instant.is_some() {
            if hold_start_instant.unwrap().elapsed() >= hold_duration_threshold {
                armed_and_ready = true;
            }
        }
    }

    execute!(stdout, Show, Clear(ClearType::All))?;
    disable_raw_mode()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut config = AppConfig::default();

    if cli.menu {
        run_menu(&mut config)?;
    }

    run_timer(config)?;

    Ok(())
}

