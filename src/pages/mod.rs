pub mod about;
pub mod annual_report;
pub mod profile;

// Re-export pages
pub use about::AboutPage;
pub use annual_report::AnnualReportPage;
// ProfilePage is no longer used - Profile tab now uses ProfileLoader
