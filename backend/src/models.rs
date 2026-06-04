use diesel::internal::derives::multiconnection::chrono;
use diesel::prelude::*;
use std::fmt;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password: String,
    pub elevated: bool,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::shelves)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
pub struct Shelf {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::books)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
pub struct Book {
    pub id: Uuid,
    pub user: Uuid,
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn13: Option<String>,
    pub isbn10: Option<String>,
    pub google_books_id: Option<String>,
    pub added_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::book_shelves)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Book))]
#[diesel(belongs_to(Shelf))]
pub struct BookShelf {
    pub book: Uuid,
    pub shelf: Uuid,
    pub added_at: chrono::NaiveDateTime,
}

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ReadingMode"]
pub enum ReadingMode {
    Pages,
    Percentage,
}

impl Display for ReadingMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ReadingMode::Pages => write!(f, "pages"),
            ReadingMode::Percentage => write!(f, "percentage"),
        }
    }
}

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ReadingGoalType"]
pub enum ReadingGoalType {
    Books,
    Pages,
}

impl Display for ReadingGoalType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ReadingGoalType::Books => write!(f, "books"),
            ReadingGoalType::Pages => write!(f, "pages"),
        }
    }
}

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ReadingGoalTimeframe"]
pub enum ReadingGoalTimeframe {
    Year,
    Month,
    Week,
}

impl Display for ReadingGoalTimeframe {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ReadingGoalTimeframe::Year => write!(f, "year"),
            ReadingGoalTimeframe::Month => write!(f, "month"),
            ReadingGoalTimeframe::Week => write!(f, "week"),
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::reading_goals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
pub struct ReadingGoal {
    pub id: Uuid,
    pub user_id: Uuid,
    pub goal_type: ReadingGoalType,
    pub timeframe: ReadingGoalTimeframe,
    pub target: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::readings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Book))]
pub struct Reading {
    pub id: Uuid,
    pub book: Uuid,
    pub user: Uuid,
    pub total_pages: i32,
    pub progress: i32,
    pub mode: ReadingMode,
    pub started_at: chrono::NaiveDate,
    pub finished_at: Option<chrono::NaiveDate>,
    pub cancelled_at: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::reading_entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Book))]
#[diesel(belongs_to(Reading))]
pub struct ReadingEntry {
    pub id: Uuid,
    pub reading: Uuid,
    pub book: Uuid,
    pub user: Uuid,
    pub progress: i32,
    pub mode: ReadingMode,
    pub read_at: chrono::NaiveDate,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
