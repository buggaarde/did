#[macro_use]
extern crate clap;
use chrono::{Duration, NaiveDate};
use clap::App;
use postgres::{Client, NoTls};
use std::{convert, error::Error, fmt};

#[derive(Debug)]
struct Note {
    id: i32,
    date: NaiveDate,
    note: String,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}: {}", self.date, self.note)
    }
}

impl convert::From<postgres::Row> for Note {
    fn from(row: postgres::Row) -> Self {
        Note {
            id: row.get(0),
            date: row.get(1),
            note: row.get(2),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(note) = matches.value_of("NOTE") {
        did(note)?;
        return Ok(());
    }

    if let Some(matches) = matches.subcommand_matches("what") {
        for row in what_rows(matches)? {
            println!("{}", Note::from(row));
        }
    }

    Ok(())
}

fn what_rows(matches: &clap::ArgMatches) -> Result<Vec<postgres::Row>, Box<dyn Error>> {
    let mut conn = connection()?;
    let today = chrono::Local::now().naive_utc().date();
    if matches.subcommand_matches("today").is_some() {
        return Ok(conn.query(
            "SELECT id, date, note FROM notes WHERE date = $1",
            &[&today],
        )?);
    }
    if matches.subcommand_matches("yesterday").is_some() {
        return Ok(conn.query(
            "SELECT id, date, note FROM notes WHERE date = $1",
            &[&today.pred()],
        )?);
    }
    if matches.subcommand_matches("week").is_some() {
        if let Some(week) = today.checked_sub_signed(Duration::days(7)) {
            return Ok(conn.query(
                "SELECT id, date, note FROM notes WHERE date >= $1",
                &[&week],
            )?);
        };
    }
    if matches.subcommand_matches("month").is_some() {
        if let Some(week) = today.checked_sub_signed(Duration::days(30)) {
            return Ok(conn.query(
                "SELECT id, date, note FROM notes WHERE date >= $1",
                &[&week],
            )?);
        };
    }
    if matches.subcommand_matches("year").is_some() {
        if let Some(week) = today.checked_sub_signed(Duration::days(365)) {
            return Ok(conn.query(
                "SELECT id, date, note FROM notes WHERE date >= $1",
                &[&week],
            )?);
        };
    }
    Ok(Vec::<postgres::Row>::new())
}

fn connection() -> Result<Client, postgres::Error> {
    Client::connect(
        "postgresql://siggaard:password@localhost:5438/database",
        NoTls,
    )
}

fn did(note: &str) -> Result<(), postgres::Error> {
    let mut conn = connection()?;
    conn.execute("INSERT INTO notes (note) VALUES ($1)", &[&note])?;
    println!("> {}", note);

    Ok(())
}
