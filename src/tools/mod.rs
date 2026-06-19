pub mod web;
pub mod academic;
pub mod docgen;
pub mod docreader;

pub use web::{WebSearchTool, WebFetchTool};
pub use academic::AcademicSearchTool;
pub use docgen::{CreateDocxTool, CreatePdfTool, CreatePptxTool};
pub use docreader::DocReaderTool;
