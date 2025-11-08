use crate::config::{self, Solves};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Print, Stylize},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::{self, Stdout};

pub fn show_stats() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let solves = config::load_solves();

    loop {
        draw_stats(&mut stdout, &solves)?;

        if let Event::Key(key_event) = event::read()? {
            if let KeyCode::Char('q') = key_event.code {
                break;
            }
        }
    }

    terminal::disable_raw_mode()?;
    stdout.execute(LeaveAlternateScreen)?;
    Ok(())
}

fn draw_stats(stdout: &mut Stdout, solves: &Solves) -> io::Result<()> {
    stdout.execute(Clear(ClearType::All))?;
    let (width, height) = terminal::size()?;

    let title = "Saved Solves";
    let title_x = (width - title.len() as u16) / 2;
    stdout
        .execute(cursor::MoveTo(title_x, 1))?
        .execute(Print(title.bold()))?;

    let instructions = "Press 'q' to quit.";
    let inst_x = (width - instructions.len() as u16) / 2;
    stdout
        .execute(cursor::MoveTo(inst_x, height - 2))?
        .execute(Print(instructions.dark_grey()))?;

    if solves.solves.is_empty() {
        let no_stats = "No solves saved yet.";
        let no_stats_x = (width - no_stats.len() as u16) / 2;
        stdout
            .execute(cursor::MoveTo(no_stats_x, height / 2))?
            .execute(Print(no_stats))?;
        return stdout.flush();
    }

    draw_table(stdout, solves, 4)?;

    stdout.flush()
}

fn draw_table(stdout: &mut Stdout, solves: &Solves, start_y: u16) -> io::Result<u16> {
    let mut y = start_y;
    let header = format!(
        "{: <25} | {: <10}",
        "Timestamp", "Time (ms)"
    );
    stdout
        .execute(cursor::MoveTo(7, y))?
        .execute(Print(header.bold()))?;
    y += 1;

    for solve in solves.solves.iter().rev() {
        let line = format!(
            "{: <25} | {: <10}",
            solve.timestamp, solve.time
        );
        stdout.execute(cursor::MoveTo(7, y))?.execute(Print(line))?;
        y += 1;
    }
    Ok(y)
}
