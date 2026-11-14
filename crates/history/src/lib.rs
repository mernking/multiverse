//! History/Undo-Redo system for Creative Engine
//!
//! Provides transactional undo/redo with grouping, branching support,
//! and persistent transaction journal.

use creative_domain::{DocumentId, ObjectId};
use creative_commands::{Command, CommandResult, BoxedCommand, UndoData};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::VecDeque;
use uuid::Uuid;

/// History errors
#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("Nothing to undo")]
    NothingToUndo,
    #[error("Nothing to redo")]
    NothingToRedo,
    #[error("Transaction not found: {0}")]
    TransactionNotFound(Uuid),
    #[error("Invalid transaction state: {0}")]
    InvalidState(String),
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Journal corruption: {0}")]
    JournalCorruption(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Transaction types for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Single atomic operation
    Single,
    /// Group of related operations (e.g., multi-object transform)
    Group,
    /// Continuous stroke input
    Stroke,
    /// Macro/script execution
    Macro,
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub document_id: DocumentId,
    pub transaction_type: TransactionType,
    pub description: String,
    pub commands: Vec<BoxedCommand>,
    pub undo_data: Vec<UndoData>,
    pub timestamp: u64,
    pub is_undone: bool,
}

/// History manager for a document
#[derive(Debug)]
pub struct History {
    document_id: DocumentId,
    undo_stack: VecDeque<Transaction>,
    redo_stack: VecDeque<Transaction>,
    max_history_size: usize,
    group_stack: Vec<Uuid>,
    journal: TransactionJournal,
}

/// Transaction journal for persistence and crash recovery
#[derive(Debug)]
pub struct TransactionJournal {
    path: std::path::PathBuf,
    max_entries: usize,
}

impl TransactionJournal {
    pub fn new(path: std::path::PathBuf, max_entries: usize) -> Self {
        std::fs::create_dir_all(&path).ok();
        Self { path, max_entries }
    }
    
    /// Record a transaction
    pub fn record(&self, transaction: &Transaction) -> Result<(), HistoryError> {
        let filename = format!("txn_{}.json", transaction.timestamp);
        let filepath = self.path.join(filename);
        
        let json = serde_json::to_string_pretty(transaction)
            .map_err(|e| HistoryError::JournalCorruption(e.to_string()))?;
        
        std::fs::write(filepath, json)
            .map_err(|e| HistoryError::JournalCorruption(e.to_string()))?;
        
        self.clean_old_entries()?;
        Ok(())
    }
    
    /// Load recent transactions
    pub fn load_recent(&self, limit: usize) -> Result<Vec<Transaction>, HistoryError> {
        let mut entries: Vec<_> = std::fs::read_dir(&self.path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("txn_"))
            .collect();
        
        entries.sort_by_key(|e| e.file_name());
        
        let mut transactions = Vec::new();
        for entry in entries.iter().rev().take(limit) {
            let content = std::fs::read_to_string(entry.path())
                .map_err(|e| HistoryError::JournalCorruption(e.to_string()))?;
            let txn: Transaction = serde_json::from_str(&content)
                .map_err(|e| HistoryError::JournalCorruption(e.to_string()))?;
            transactions.push(txn);
        }
        
        Ok(transactions)
    }
    
    /// Clean old journal entries
    fn clean_old_entries(&self) -> Result<(), HistoryError> {
        let mut entries: Vec<_> = std::fs::read_dir(&self.path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("txn_"))
            .collect();
        
        entries.sort_by_key(|e| e.file_name());
        
        if entries.len() > self.max_entries {
            for entry in entries[..entries.len() - self.max_entries].iter() {
                std::fs::remove_file(entry.path()).ok();
            }
        }
        
        Ok(())
    }
    
    /// Clear journal
    pub fn clear(&self) -> Result<(), HistoryError> {
        let entries: Vec<_> = std::fs::read_dir(&self.path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("txn_"))
            .collect();
        
        for entry in entries {
            std::fs::remove_file(entry.path()).ok();
        }
        
        Ok(())
    }
}

impl History {
    /// Create a new history manager
    pub fn new(document_id: DocumentId, journal_path: std::path::PathBuf, max_history_size: usize) -> Self {
        Self {
            document_id,
            undo_stack: VecDeque::with_capacity(max_history_size),
            redo_stack: VecDeque::with_capacity(max_history_size),
            max_history_size,
            group_stack: Vec::new(),
            journal: TransactionJournal::new(journal_path, max_history_size * 2),
        }
    }
    
    /// Execute a command and record it in history
    pub fn execute(
        &mut self,
        command: BoxedCommand,
    ) -> Result<CommandResult, HistoryError> {
        // Validate command
        command.validate().map_err(|e| HistoryError::CommandFailed(e.to_string()))?;
        
        // Check document ID matches
        if command.document_id() != self.document_id {
            return Err(HistoryError::InvalidState("Command document ID mismatch".to_string()));
        }
        
        // Execute command
        let result = command.execute();
        
        // If successful, record in history
        if result.success {
            let undo_data = result.undo_data.clone().unwrap_or(UndoData::CreateObject { object_id: ObjectId::new() });
            self.record_transaction(
                TransactionType::Single,
                &format!("{:?}", command),
                vec![command],
                vec![undo_data],
            );
        }
        
        // Clear redo stack on new action
        self.redo_stack.clear();
        
        Ok(result)
    }
    
    /// Execute a batch of commands as a single transaction
    pub fn execute_batch(
        &mut self,
        commands: Vec<BoxedCommand>,
        description: String,
        transaction_type: TransactionType,
    ) -> Result<Vec<CommandResult>, HistoryError> {
        if commands.is_empty() {
            return Ok(Vec::new());
        }
        
        // Validate all commands
        for cmd in &commands {
            cmd.validate().map_err(|e| HistoryError::CommandFailed(e.to_string()))?;
            if cmd.document_id() != self.document_id {
                return Err(HistoryError::InvalidState("Command document ID mismatch".to_string()));
            }
        }
        
        // Execute all commands
        let mut results = Vec::new();
        let mut all_undo_data = Vec::new();
        
        for cmd in &commands {
            let result = cmd.execute();
            
            // Use undo data if available, otherwise create placeholder
            let undo_data = result.undo_data.clone().unwrap_or(UndoData::CreateObject { object_id: ObjectId::new() });
            all_undo_data.push(undo_data);
            results.push(result);
        }
        
        // Record as single transaction if all commands succeeded
        if results.iter().all(|r| r.success) {
            self.record_transaction(transaction_type, &description, commands, all_undo_data);
        }
        
        // Clear redo stack
        self.redo_stack.clear();
        
        Ok(results)
    }
    
    /// Record a transaction in the undo stack
    fn record_transaction(
        &mut self,
        transaction_type: TransactionType,
        description: &str,
        commands: Vec<BoxedCommand>,
        undo_data: Vec<UndoData>,
    ) {
        let transaction = Transaction {
            id: Uuid::new_v4(),
            document_id: self.document_id,
            transaction_type,
            description: description.to_string(),
            commands,
            undo_data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            is_undone: false,
        };
        
        // Record in journal
        self.journal.record(&transaction).ok();
        
        // Add to undo stack
        self.undo_stack.push_back(transaction);
        
        // Trim stack if needed
        while self.undo_stack.len() > self.max_history_size {
            self.undo_stack.pop_front();
        }
    }
    
    /// Undo the last transaction
    pub fn undo(&mut self) -> Result<Transaction, HistoryError> {
        let mut transaction = self.undo_stack.pop_back()
            .ok_or(HistoryError::NothingToUndo)?;
        
        // Mark as undone
        transaction.is_undone = true;
        
        // Add to redo stack
        self.redo_stack.push_back(transaction.clone());
        
        // Trim redo stack
        while self.redo_stack.len() > self.max_history_size {
            self.redo_stack.pop_front();
        }
        
        Ok(transaction)
    }
    
    /// Redo the last undone transaction
    pub fn redo(&mut self) -> Result<Transaction, HistoryError> {
        let mut transaction = self.redo_stack.pop_back()
            .ok_or(HistoryError::NothingToRedo)?;
        
        // Mark as not undone
        transaction.is_undone = false;
        
        // Add back to undo stack
        self.undo_stack.push_back(transaction.clone());
        
        // Trim undo stack
        while self.undo_stack.len() > self.max_history_size {
            self.undo_stack.pop_front();
        }
        
        Ok(transaction)
    }
    
    /// Begin a transaction group
    pub fn begin_group(&mut self, _description: &str) -> Uuid {
        let group_id = Uuid::new_v4();
        self.group_stack.push(group_id);
        group_id
    }
    
    /// End a transaction group
    pub fn end_group(&mut self) -> Option<Uuid> {
        self.group_stack.pop()
    }
    
    /// Get current group ID
    pub fn current_group(&self) -> Option<Uuid> {
        self.group_stack.last().copied()
    }
    
    /// Get group depth
    pub fn group_depth(&self) -> usize {
        self.group_stack.len()
    }
    
    /// Check if can undo
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    /// Check if can redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    /// Get undo stack size
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
    
    /// Get redo stack size
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
    
    /// Get last undo description
    pub fn last_undo_description(&self) -> Option<&str> {
        self.undo_stack.back().map(|t| t.description.as_str())
    }
    
    /// Get last redo description
    pub fn last_redo_description(&self) -> Option<&str> {
        self.redo_stack.back().map(|t| t.description.as_str())
    }
    
    /// Get full undo history (for UI)
    pub fn undo_history(&self) -> Vec<&Transaction> {
        self.undo_stack.iter().rev().collect()
    }
    
    /// Get full redo history (for UI)
    pub fn redo_history(&self) -> Vec<&Transaction> {
        self.redo_stack.iter().rev().collect()
    }
    
    /// Clear history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.journal.clear().ok();
    }
    
    /// Get journal reference
    pub fn journal(&self) -> &TransactionJournal {
        &self.journal
    }
    
    /// Load history from journal (for recovery)
    pub fn load_from_journal(&mut self, limit: usize) -> Result<usize, HistoryError> {
        let transactions = self.journal.load_recent(limit)?;
        let count = transactions.len();
        
        for txn in transactions {
            if txn.is_undone {
                self.redo_stack.push_back(txn);
            } else {
                self.undo_stack.push_back(txn);
            }
        }
        
        Ok(count)
    }
}

/// Extension trait for Command to get description
pub trait CommandExt {
    fn description(&self) -> String;
}

impl<C: Command> CommandExt for C {
    fn description(&self) -> String {
        format!("{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{DocumentId, ObjectId, LayerId, Color, Transform2D, Point2, Rect, BlendMode};
    use creative_commands::{DocumentCommand, DocumentSettings, ObjectCommand, ObjectType, ObjectStyle, GeometryData};
    use creative_geometry::{Geometry, ShapeGeometry, GeometryId};
    
    use tempfile::TempDir;
    use uuid::Uuid;
    
    fn create_test_document_command(doc_id: DocumentId) -> DocumentCommand {
        DocumentCommand::CreateDocument {
            document_id: doc_id,
            name: "Test".to_string(),
            settings: DocumentSettings::default(),
        }
    }
    
    fn create_test_object_command() -> ObjectCommand {
        let obj_id = ObjectId::new();
        let layer_id = LayerId::new();
        let _geom_id = GeometryId::new();
        
        let rect = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        let shape = ShapeGeometry::new(creative_geometry::ShapeType::Rectangle, rect);
        let _geometry = Geometry::Shape(shape);
        
        ObjectCommand::CreateObject {
            document_id: DocumentId::new(),
            object_id: obj_id,
            layer_id,
            object_type: ObjectType::Shape,
            transform: Transform2D::IDENTITY,
            style: ObjectStyle {
                fill: Some(Color::RED),
                stroke: Some(Color::BLACK),
                stroke_width: 2.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            },
            geometry: GeometryData::Shape {
                shape_type: creative_commands::ShapeType::Rectangle,
                path_data: vec![],
            },
            z_index: 0,
        }
    }
    
    #[test]
    fn test_history_creation() {
        let temp_dir = TempDir::new().unwrap();
        let history = History::new(DocumentId::new(), temp_dir.path().to_path_buf(), 100);
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }
    
    #[test]
    fn test_undo_redo_single_command() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
        
        let cmd = BoxedCommand::Document(create_test_document_command(doc_id));
        let result = history.execute(cmd).unwrap();
        
        assert!(result.success);
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
        
        let undone = history.undo().unwrap();
        // After undo, the transaction is moved to redo stack, undo stack is empty
        assert_eq!(undone.id, history.redo_stack.back().map(|t| t.id).unwrap_or(Uuid::nil()));
        
        assert!(!history.can_undo());
        assert!(history.can_redo());
        assert_eq!(history.redo_count(), 1);
        
        let redone = history.redo().unwrap();
        // After redo, the transaction is moved back to undo stack
        assert_eq!(redone.id, history.undo_stack.back().map(|t| t.id).unwrap_or(Uuid::nil()));
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }
    
    #[test]
    fn test_batch_execution() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
        
        let cmd1 = BoxedCommand::Document(create_test_document_command(doc_id));
        let cmd2 = BoxedCommand::Document(DocumentCommand::RenameDocument {
            document_id: doc_id,
            new_name: "Renamed".to_string(),
        });
        
        let results = history.execute_batch(
            vec![cmd1, cmd2],
            "Create and rename".to_string(),
            TransactionType::Group,
        ).unwrap();
        
        assert_eq!(results.len(), 2);
        assert!(history.can_undo());
        assert_eq!(history.undo_count(), 1); // Single transaction
        
        let undone = history.undo().unwrap();
        assert_eq!(undone.transaction_type, TransactionType::Group);
        assert_eq!(undone.commands.len(), 2);
    }
    
    #[test]
    fn test_redo_cleared_on_new_action() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
        
        // Execute and undo
        let cmd = BoxedCommand::Document(create_test_document_command(doc_id));
        history.execute(cmd).unwrap();
        history.undo().unwrap();
        
        assert!(history.can_redo());
        
        // Execute new command - should clear redo
        let cmd2 = BoxedCommand::Document(DocumentCommand::RenameDocument {
            document_id: doc_id,
            new_name: "New Name".to_string(),
        });
        history.execute(cmd2).unwrap();
        
        assert!(!history.can_redo());
        assert!(history.can_undo());
    }
    
    #[test]
    fn test_group_operations() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
        
        let group_id = history.begin_group("Test Group");
        assert_eq!(history.group_depth(), 1);
        assert_eq!(history.current_group(), Some(group_id));
        
        history.end_group();
        assert_eq!(history.group_depth(), 0);
        assert_eq!(history.current_group(), None);
    }
    
    #[test]
    fn test_nested_groups() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
        
        let outer = history.begin_group("Outer");
        assert_eq!(history.group_depth(), 1);
        
        let inner = history.begin_group("Inner");
        assert_eq!(history.group_depth(), 2);
        assert_eq!(history.current_group(), Some(inner));
        
        history.end_group();
        assert_eq!(history.group_depth(), 1);
        assert_eq!(history.current_group(), Some(outer));
        
        history.end_group();
        assert_eq!(history.group_depth(), 0);
        assert_eq!(history.current_group(), None);
    }
    
    #[test]
    fn test_history_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        
        // Create history and execute commands
        {
            let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
            let cmd = BoxedCommand::Document(create_test_document_command(doc_id));
            history.execute(cmd).unwrap();
        }
        
        // Create new history and load from journal
        {
            let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
            let loaded = history.load_from_journal(10).unwrap();
            
            assert_eq!(loaded, 1);
            assert!(history.can_undo());
        }
    }
    
    #[test]
    fn test_clear_history() {
        let temp_dir = TempDir::new().unwrap();
        let doc_id = DocumentId::new();
        let mut history = History::new(doc_id, temp_dir.path().to_path_buf(), 100);
        
        let cmd = BoxedCommand::Document(create_test_document_command(doc_id));
        history.execute(cmd).unwrap();
        
        assert!(history.can_undo());
        
        history.clear();
        
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}